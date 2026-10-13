//! Cinematic intro between the title screen and the first card: six
//! pixel-art shots (see `intro_art`) with typed captions underneath.
//! Click, Space or Enter finishes the caption or moves on; Escape skips it all.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, EventTarget, HtmlElement, KeyboardEvent};

use crate::audio::Music;
use crate::intro_art::{spark, Art, Shot, LIGHTNING};
use crate::view::{el, window};

const CHAR_MS: f64 = 38.0;
const LINE_PAUSE_MS: f64 = 900.0;
/// How long a finished caption stays up before the next shot.
const HOLD_MS: f64 = 3800.0;
const FINAL_HOLD_MS: f64 = 5500.0;
/// Fade through black between shots. Keep in sync with `.intro-caption` in style.css.
const FADE_MS: f64 = 900.0;
/// Fade out of the last shot into the game.
const END_FADE_MS: f64 = 1800.0;

struct Scene {
    shot: Shot,
    lines: &'static [&'static str],
    /// When the caption starts typing, in ms after the shot begins.
    text_at: f64,
    /// Hard cut from the previous shot instead of a fade.
    cut_in: bool,
}

/// The last line of the last scene is the hook: it's styled apart and lands on a boom.
const SCENES: [Scene; 6] = [
    Scene {
        shot: Shot::City,
        lines: &[
            "October 13th. It had been raining for eleven days straight.",
            "That morning, a crime shocked the whole country.",
        ],
        text_at: 1500.0,
        cut_in: false,
    },
    Scene {
        shot: Shot::Hotel,
        lines: &[
            "The Hotel Lumen. Old, huge, cheap. Nobody asks questions.",
            "Two guests checked into room 1313. Then they vanished.",
        ],
        text_at: 1200.0,
        cut_in: false,
    },
    Scene {
        shot: Shot::Kindle,
        lines: &[
            "All they left behind was a broken e-reader.",
            "Its screen would not turn off. It showed only this.",
        ],
        text_at: 1800.0,
        cut_in: false,
    },
    Scene {
        shot: Shot::Room,
        lines: &[
            "The room was torn apart. The air reeked of sulfur.",
            "Every device was fried. No light would come on.",
        ],
        text_at: 1500.0,
        cut_in: false,
    },
    Scene {
        shot: Shot::Cctv,
        lines: &[
            "Hours earlier, a street camera caught the man who rented it.",
            "He kept looking back over his shoulder.",
        ],
        text_at: 1000.0,
        cut_in: false,
    },
    Scene {
        shot: Shot::Office,
        lines: &["The case lands on your desk. Your first.", "What happened in room 1313?"],
        text_at: 5400.0,
        cut_in: true,
    },
];

#[derive(Clone, Copy, PartialEq)]
enum Leave {
    Next,
    End,
}

struct Intro {
    doc: Document,
    art: Art,
    root: HtmlElement,
    caption: HtmlElement,
    /// (typed, not yet typed) spans per caption line; the untyped part is
    /// invisible but keeps the layout from shifting while typing.
    lines: Vec<(HtmlElement, HtmlElement)>,
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
    /// Whether the TV in the room shot was sparking last frame (crackle on each new burst).
    sparking: bool,
    leaving: Option<(f64, Leave)>,
    on_done: Option<Box<dyn FnOnce()>>,
}

type Shared = Rc<RefCell<Intro>>;

/// Plays the intro, then calls `on_done`. If it can't be built, `on_done` runs right away.
pub fn play(root_id: &str, on_done: impl FnOnce() + 'static) {
    match build(root_id) {
        Ok(intro) => {
            intro.borrow_mut().on_done = Some(Box::new(on_done));
            run(intro);
        }
        Err(e) => {
            web_sys::console::error_1(&e);
            on_done();
        }
    }
}

fn build(root_id: &str) -> Result<Shared, JsValue> {
    let doc = window().document().ok_or("no document")?;
    let host = doc
        .get_element_by_id(root_id)
        .ok_or_else(|| format!("missing #{root_id} element"))?;

    let root = el(&doc, "div", "intro")?;
    let (art, canvas) = Art::new(&doc)?;
    canvas.set_class_name("intro-screen");
    let caption = el(&doc, "div", "intro-caption")?;
    let hint = el(&doc, "p", "intro-hint")?;
    hint.set_text_content(Some("click to continue · esc to skip"));
    root.append_child(&canvas)?;
    root.append_child(&caption)?;
    root.append_child(&hint)?;
    let grain = el(&doc, "div", "intro-grain")?;
    root.append_child(&grain)?;
    host.append_child(&root)?;

    // Created after the title screen's gesture, so the browser lets it play.
    let music = Music::new().map_err(|e| web_sys::console::warn_1(&e)).ok();
    if let Some(m) = &music {
        m.drone();
    }

    let intro = Rc::new(RefCell::new(Intro {
        doc: doc.clone(),
        art,
        root: root.clone(),
        caption,
        lines: Vec::new(),
        music,
        scene: 0,
        now: 0.0,
        began: None,
        hurry: 0.0,
        shown: 0,
        boomed: false,
        sparking: false,
        leaving: None,
        on_done: None,
    }));
    intro.borrow_mut().enter(0)?;

    on(&root.into(), "click", &intro, |s, _| s.borrow_mut().advance())?;
    on(&doc.into(), "keydown", &intro, |s, ev| {
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
    Ok(intro)
}

impl Intro {
    fn enter(&mut self, i: usize) -> Result<(), JsValue> {
        self.scene = i;
        self.began = None;
        self.hurry = 0.0;
        self.shown = 0;
        self.boomed = false;
        self.leaving = None;

        self.caption.set_inner_html("");
        self.caption.class_list().remove_1("hide")?;
        self.lines.clear();
        let scene = &SCENES[i];
        for (n, text) in scene.lines.iter().enumerate() {
            let accent = i == SCENES.len() - 1 && n == scene.lines.len() - 1;
            let line = el(&self.doc, "p", if accent { "intro-line accent" } else { "intro-line" })?;
            let typed = el(&self.doc, "span", "")?;
            let rest = el(&self.doc, "span", "rest")?;
            rest.set_text_content(Some(text));
            line.append_child(&typed)?;
            line.append_child(&rest)?;
            self.caption.append_child(&line)?;
            self.lines.push((typed, rest));
        }
        self.cue(scene.shot);
        Ok(())
    }

    fn cue(&mut self, shot: Shot) {
        let Some(m) = &mut self.music else { return };
        match shot {
            Shot::City => {
                m.rain(0.35, 2600.0, 2.0);
                for s in LIGHTNING {
                    m.thunder(s + 0.6);
                }
            }
            Shot::Hotel => m.rain(0.3, 2200.0, 1.0),
            Shot::Kindle => {
                m.rain(0.1, 500.0, 1.5);
                m.sting();
            }
            Shot::Room => m.rain(0.14, 700.0, 1.5),
            Shot::Cctv => {
                m.rain(0.0, 400.0, 0.3);
                m.hum(0.05, 0.3);
                m.static_burst();
            }
            Shot::Office => {
                m.hum(0.015, 3.0);
                m.rain(0.1, 900.0, 3.0);
            }
        }
    }

    /// Draws one frame. Returns false once the intro is over.
    fn frame(&mut self, now: f64) -> bool {
        self.now = now;
        let began = *self.began.get_or_insert(now);
        let scene = &SCENES[self.scene];
        let elapsed = now - began;
        self.art.draw(scene.shot, elapsed / 1000.0);
        let sparking = scene.shot == Shot::Room && spark(elapsed / 1000.0).is_some();
        if sparking && !self.sparking {
            if let Some(m) = &self.music {
                m.crackle();
            }
        }
        self.sparking = sparking;
        let clock = self.caption_clock();
        self.type_caption(clock);

        let mut black = if scene.cut_in { 0.0 } else { 1.0 - elapsed / FADE_MS };
        match self.leaving {
            Some((since, leave)) => {
                let span = match leave {
                    Leave::Next if SCENES[self.scene + 1].cut_in => 0.0,
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
                let last = self.scene == SCENES.len() - 1;
                let hold = if last { FINAL_HOLD_MS } else { HOLD_MS };
                if clock > caption_ms(scene.lines) + hold {
                    self.leave(if last { Leave::End } else { Leave::Next });
                }
            }
        }
        self.art.fade(black);
        true
    }

    fn caption_clock(&self) -> f64 {
        match self.began {
            Some(began) => self.now - began - SCENES[self.scene].text_at + self.hurry,
            None => f64::NEG_INFINITY,
        }
    }

    fn type_caption(&mut self, clock: f64) {
        let scene = &SCENES[self.scene];
        let mut counts = Vec::with_capacity(scene.lines.len());
        let mut start = 0.0;
        for line in scene.lines {
            let n = line.chars().count();
            counts.push((((clock - start) / CHAR_MS).floor().max(0.0) as usize).min(n));
            start += n as f64 * CHAR_MS + LINE_PAUSE_MS;
        }
        let total: usize = counts.iter().sum();
        if total == self.shown {
            return;
        }
        for ((typed, rest), (line, &k)) in self.lines.iter().zip(scene.lines.iter().zip(&counts)) {
            let split = line.char_indices().nth(k).map_or(line.len(), |(b, _)| b);
            typed.set_text_content(Some(&line[..split]));
            rest.set_text_content(Some(&line[split..]));
        }
        self.shown = total;

        let Some(m) = &self.music else { return };
        m.tick();
        let accent_started = counts.last().is_some_and(|&k| k > 0);
        if self.scene == SCENES.len() - 1 && accent_started && !self.boomed {
            self.boomed = true;
            m.boom();
        }
    }

    /// Finishes the caption if it's still typing, otherwise moves on.
    fn advance(&mut self) {
        if self.leaving.is_some() || self.began.is_none() {
            return;
        }
        let behind = caption_ms(SCENES[self.scene].lines) - self.caption_clock();
        if behind > 0.0 {
            self.hurry += behind;
        } else if self.scene == SCENES.len() - 1 {
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

/// Drives `frame` with requestAnimationFrame until the intro ends, then hands over.
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
        // Handlers outlive the intro; ignore input once it has handed over.
        if s.borrow().on_done.is_some() {
            handler(&s, ev);
        }
    });
    target.add_event_listener_with_callback(ty, cb.as_ref().unchecked_ref())?;
    cb.forget();
    Ok(())
}
