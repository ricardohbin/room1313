//! Cinematic player: a reel of pixel-art shots (see `art`) with typed
//! captions underneath. Used by the intro and the endings.
//! Click, Space or Enter finishes the caption or moves on; Escape skips it all.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, EventTarget, HtmlElement, KeyboardEvent};

use crate::art::{Art, Shot};
use crate::audio::Music;
use crate::i18n::t;
use crate::view::{el, window};

const CHAR_MS: f64 = 38.0;
const LINE_PAUSE_MS: f64 = 900.0;
/// How long a finished caption stays up before the next shot.
const HOLD_MS: f64 = 3800.0;
const FINAL_HOLD_MS: f64 = 5500.0;
/// Fade through black between shots. Keep in sync with `.cine-caption` in style.css.
const FADE_MS: f64 = 900.0;
/// Fade out of the last shot.
const END_FADE_MS: f64 = 1800.0;
const HINT: &str = "click to continue · esc to skip";

#[cfg(test)]
pub(crate) fn texts() -> [&'static str; 1] {
    [HINT]
}

/// A whole cinematic. The last line of the last scene is the hook: it's
/// styled apart and lands on a boom.
pub struct Film {
    pub scenes: &'static [Scene],
    /// Starts the music, once, when the film begins.
    pub score: fn(&mut Music),
}

impl Film {
    /// Every caption line, in English.
    #[cfg(test)]
    pub fn lines(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.scenes.iter().flat_map(|s| s.lines.iter().copied())
    }
}

pub struct Scene {
    shot: Shot,
    lines: &'static [&'static str],
    /// When the caption starts typing, in ms after the shot begins.
    text_at: f64,
    /// Hard cut from the previous shot instead of a fade.
    cut_in: bool,
    /// Sounds for when the shot starts.
    cue: fn(&mut Music),
    /// Called every frame with the shot's previous and current time (s),
    /// to sync sounds to the picture: fire when `prev < moment <= now`.
    sync: fn(&mut Music, f64, f64),
}

impl Scene {
    pub const fn new(shot: Shot, lines: &'static [&'static str], text_at: f64) -> Self {
        Self { shot, lines, text_at, cut_in: false, cue: |_| {}, sync: |_, _, _| {} }
    }

    pub const fn cut_in(mut self) -> Self {
        self.cut_in = true;
        self
    }

    pub const fn cue(mut self, cue: fn(&mut Music)) -> Self {
        self.cue = cue;
        self
    }

    pub const fn sync(mut self, sync: fn(&mut Music, f64, f64)) -> Self {
        self.sync = sync;
        self
    }
}

/// True on the frame where the shot's clock crosses `moment` (seconds).
pub fn crossed(prev: f64, now: f64, moment: f64) -> bool {
    prev < moment && moment <= now
}

#[derive(Clone, Copy, PartialEq)]
enum Leave {
    Next,
    End,
}

struct Player {
    film: &'static Film,
    doc: Document,
    art: Art,
    root: HtmlElement,
    caption: HtmlElement,
    /// (typed, not yet typed) spans per caption line; the untyped part is
    /// invisible but keeps the layout from shifting while typing.
    lines: Vec<(HtmlElement, HtmlElement)>,
    /// The current shot's caption, in the player's language.
    text: Vec<&'static str>,
    music: Option<Music>,
    scene: usize,
    /// Timestamp of the latest frame, and of the current shot's first frame.
    now: f64,
    began: Option<f64>,
    /// Extra ms on the caption clock when the player hurries it along.
    hurry: f64,
    /// Characters revealed so far, to click once per new letter.
    shown: usize,
    boomed: bool,
    /// The shot's clock (s) at the previous frame, for `Scene::sync`.
    prev_t: f64,
    leaving: Option<(f64, Leave)>,
    on_done: Option<Box<dyn FnOnce()>>,
}

type Shared = Rc<RefCell<Player>>;

/// Plays `film`, then calls `on_done`. If it can't be built, `on_done` runs right away.
pub fn play(film: &'static Film, root_id: &str, on_done: impl FnOnce() + 'static) {
    match build(film, root_id) {
        Ok(player) => {
            player.borrow_mut().on_done = Some(Box::new(on_done));
            run(player);
        }
        Err(e) => {
            web_sys::console::error_1(&e);
            on_done();
        }
    }
}

fn build(film: &'static Film, root_id: &str) -> Result<Shared, JsValue> {
    let doc = window().document().ok_or("no document")?;
    let host = doc
        .get_element_by_id(root_id)
        .ok_or_else(|| format!("missing #{root_id} element"))?;

    let root = el(&doc, "div", "cine")?;
    let (art, canvas) = Art::new(&doc)?;
    canvas.set_class_name("cine-screen");
    let caption = el(&doc, "div", "cine-caption")?;
    let hint = el(&doc, "p", "cine-hint")?;
    hint.set_text_content(Some(t(HINT)));
    root.append_child(&canvas)?;
    root.append_child(&caption)?;
    root.append_child(&hint)?;
    let grain = el(&doc, "div", "cine-grain")?;
    root.append_child(&grain)?;
    host.append_child(&root)?;

    // Created after the title screen's gesture, so the browser lets it play.
    let mut music = Music::new().map_err(|e| web_sys::console::warn_1(&e)).ok();
    if let Some(m) = &mut music {
        (film.score)(m);
    }

    let player = Rc::new(RefCell::new(Player {
        film,
        doc: doc.clone(),
        art,
        root: root.clone(),
        caption,
        lines: Vec::new(),
        text: Vec::new(),
        music,
        scene: 0,
        now: 0.0,
        began: None,
        hurry: 0.0,
        shown: 0,
        boomed: false,
        prev_t: 0.0,
        leaving: None,
        on_done: None,
    }));
    player.borrow_mut().enter(0)?;

    on(&root.into(), "click", &player, |s, _| s.borrow_mut().advance())?;
    on(&doc.into(), "keydown", &player, |s, ev| {
        let ev: KeyboardEvent = ev.unchecked_into();
        if ev.repeat() {
            return;
        }
        match ev.key().as_str() {
            "Escape" => s.borrow_mut().leave(Leave::End),
            " " | "Enter" | "ArrowRight" => {
                ev.prevent_default();
                s.borrow_mut().advance();
            }
            _ => {}
        }
    })?;
    Ok(player)
}

impl Player {
    fn enter(&mut self, i: usize) -> Result<(), JsValue> {
        self.scene = i;
        self.began = None;
        self.hurry = 0.0;
        self.shown = 0;
        self.boomed = false;
        self.leaving = None;
        self.prev_t = 0.0;

        self.caption.set_inner_html("");
        self.caption.class_list().remove_1("hide")?;
        self.lines.clear();
        let scene = &self.film.scenes[i];
        self.text = scene.lines.iter().map(|&line| t(line)).collect();
        for (n, text) in self.text.iter().enumerate() {
            let accent = i == self.film.scenes.len() - 1 && n == self.text.len() - 1;
            let line = el(&self.doc, "p", if accent { "cine-line accent" } else { "cine-line" })?;
            let typed = el(&self.doc, "span", "")?;
            let rest = el(&self.doc, "span", "rest")?;
            rest.set_text_content(Some(text));
            line.append_child(&typed)?;
            line.append_child(&rest)?;
            self.caption.append_child(&line)?;
            self.lines.push((typed, rest));
        }
        if let Some(m) = &mut self.music {
            (scene.cue)(m);
        }
        Ok(())
    }

    /// Draws one frame. Returns false once the film is over.
    fn frame(&mut self, now: f64) -> bool {
        self.now = now;
        let began = *self.began.get_or_insert(now);
        let scene = &self.film.scenes[self.scene];
        let elapsed = now - began;
        let t = elapsed / 1000.0;
        self.art.draw(scene.shot, t);
        if let Some(m) = &mut self.music {
            (scene.sync)(m, self.prev_t, t);
        }
        self.prev_t = t;
        let clock = self.caption_clock();
        self.type_caption(clock);

        let mut black = if scene.cut_in { 0.0 } else { 1.0 - elapsed / FADE_MS };
        match self.leaving {
            Some((since, leave)) => {
                let span = match leave {
                    Leave::Next if self.film.scenes[self.scene + 1].cut_in => 0.0,
                    Leave::Next => FADE_MS,
                    Leave::End => END_FADE_MS,
                };
                let k = if span > 0.0 { (now - since) / span } else { 1.0 };
                black = black.max(k);
                if k >= 1.0 {
                    match leave {
                        Leave::Next => {
                            if let Err(e) = self.enter(self.scene + 1) {
                                web_sys::console::error_1(&e);
                            }
                        }
                        Leave::End => {
                            self.root.remove();
                            return false;
                        }
                    }
                }
            }
            None => {
                let last = self.scene == self.film.scenes.len() - 1;
                let hold = if last { FINAL_HOLD_MS } else { HOLD_MS };
                if clock > caption_ms(&self.text) + hold {
                    self.leave(if last { Leave::End } else { Leave::Next });
                }
            }
        }
        self.art.fade(black);
        true
    }

    fn caption_clock(&self) -> f64 {
        match self.began {
            Some(began) => self.now - began - self.film.scenes[self.scene].text_at + self.hurry,
            None => f64::NEG_INFINITY,
        }
    }

    fn type_caption(&mut self, clock: f64) {
        let mut counts = Vec::with_capacity(self.text.len());
        let mut start = 0.0;
        for line in &self.text {
            let n = line.chars().count();
            counts.push((((clock - start) / CHAR_MS).floor().max(0.0) as usize).min(n));
            start += n as f64 * CHAR_MS + LINE_PAUSE_MS;
        }
        let total: usize = counts.iter().sum();
        if total == self.shown {
            return;
        }
        for ((typed, rest), (line, &k)) in self.lines.iter().zip(self.text.iter().zip(&counts)) {
            let split = line.char_indices().nth(k).map_or(line.len(), |(b, _)| b);
            typed.set_text_content(Some(&line[..split]));
            rest.set_text_content(Some(&line[split..]));
        }
        self.shown = total;

        let Some(m) = &self.music else { return };
        m.tick();
        let accent_started = counts.last().is_some_and(|&k| k > 0);
        if self.scene == self.film.scenes.len() - 1 && accent_started && !self.boomed {
            self.boomed = true;
            m.boom();
        }
    }

    /// Finishes the caption if it's still typing, otherwise moves on.
    fn advance(&mut self) {
        if self.leaving.is_some() || self.began.is_none() {
            return;
        }
        let behind = caption_ms(&self.text) - self.caption_clock();
        if behind > 0.0 {
            self.hurry += behind;
        } else if self.scene == self.film.scenes.len() - 1 {
            self.leave(Leave::End);
        } else {
            self.leave(Leave::Next);
        }
    }

    fn leave(&mut self, leave: Leave) {
        if self.leaving.is_some_and(|(_, l)| l == Leave::End) {
            return;
        }
        self.leaving = Some((self.now, leave));
        let _ = self.caption.class_list().add_1("hide");
        if leave == Leave::End {
            let _ = self.root.class_list().add_1("leaving");
            if let Some(m) = self.music.take() {
                m.fade_out(END_FADE_MS / 1000.0);
            }
        }
    }
}

/// Total typing time of a caption, pauses included.
fn caption_ms(lines: &[&str]) -> f64 {
    let chars: usize = lines.iter().map(|l| l.chars().count()).sum();
    chars as f64 * CHAR_MS + lines.len().saturating_sub(1) as f64 * LINE_PAUSE_MS
}

/// Drives `frame` with requestAnimationFrame until the film ends, then hands over.
fn run(s: Shared) {
    type Loop = Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>;
    let f: Loop = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::new(move |now: f64| {
        if s.borrow_mut().frame(now) {
            if let Some(cb) = f.borrow().as_ref() {
                request_frame(cb);
            }
            return;
        }
        let done = s.borrow_mut().on_done.take();
        if let Some(done) = done {
            done();
        }
        // Drop the loop's closure so it gets cleaned up once it returns.
        let _ = f.borrow_mut().take();
    }));
    if let Some(cb) = g.borrow().as_ref() {
        request_frame(cb);
    };
}

fn request_frame(cb: &Closure<dyn FnMut(f64)>) {
    let _ = window().request_animation_frame(cb.as_ref().unchecked_ref());
}

fn on(
    target: &EventTarget,
    ty: &str,
    s: &Shared,
    handler: impl Fn(&Shared, web_sys::Event) + 'static,
) -> Result<(), JsValue> {
    let s = s.clone();
    let cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |ev| {
        // Handlers outlive the film; ignore input once it has handed over.
        if s.borrow().on_done.is_some() {
            handler(&s, ev);
        }
    });
    target.add_event_listener_with_callback(ty, cb.as_ref().unchecked_ref())?;
    cb.forget();
    Ok(())
}
