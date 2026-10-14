//! Title screen, mounted when a language is picked (`language.rs`); that click
//! or key press is the gesture browsers want before they play audio. Phases:
//! 1. Intro: black hold with a rising rumble, then ROOM 1313 flickers in on a boom.
//! 2. Menu: the theme loops; NEW GAME fades everything to black and hands over to the game.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, EventTarget, HtmlElement, KeyboardEvent};

use crate::audio::Music;
use crate::i18n::t;
use crate::language;
use crate::store;
use crate::view::{after, el, window};

// Intro timeline. Keep in sync with the matching transitions/animations in style.css.
const BLACK_HOLD_MS: i32 = 2200;
const TITLE_FLICKER_MS: i32 = 2600;
const LEAVE_MS: i32 = 1800;
const WHISPER_MS: i32 = 2600;
const WHISPER: &str = "There is no exit.";

#[derive(Clone, Copy, PartialEq)]
enum Item {
    NewGame,
    Continue,
    Options,
    Exit,
}

impl Item {
    const ALL: [Item; 4] = [Item::NewGame, Item::Continue, Item::Options, Item::Exit];

    fn label(self) -> &'static str {
        match self {
            Item::NewGame => "New Game",
            Item::Continue => "Continue",
            Item::Options => crate::options::TITLE,
            Item::Exit => "Exit",
        }
    }

    fn text(self) -> &'static str {
        t(self.label())
    }

    /// Continue stays disabled until there is a saved game.
    fn enabled(self, saved: bool) -> bool {
        saved || self != Item::Continue
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Intro,
    Menu,
    Leaving,
}

struct Splash {
    phase: Phase,
    root: HtmlElement,
    menu: HtmlElement,
    items: Vec<HtmlElement>,
    whisper: HtmlElement,
    selected: usize,
    music: Option<Music>,
    /// Whether there is a saved game to continue.
    saved: bool,
    on_start: Option<Box<dyn FnOnce(Start)>>,
}

type Shared = Rc<RefCell<Splash>>;

/// Call it from inside a click or key handler, so the music is allowed to start.
/// How the player left the title screen.
#[derive(Clone, Copy, PartialEq)]
pub enum Start {
    NewGame,
    Continue,
}

pub fn mount(root_id: &str, on_start: impl FnOnce(Start) + 'static) -> Result<(), JsValue> {
    let document = window().document().ok_or("no document")?;
    let host = document
        .get_element_by_id(root_id)
        .ok_or_else(|| format!("missing #{root_id} element"))?;
    let splash = Rc::new(RefCell::new(Splash::build(&document, &host, Box::new(on_start))?));

    begin_intro(&splash);
    on(&document.into(), "keydown", &splash, |s, ev| on_key(s, ev.unchecked_into()))?;

    let items = splash.borrow().items.clone();
    for (i, item) in items.iter().enumerate() {
        let target: &EventTarget = item;
        on(target, "pointerenter", &splash, move |s, _| select(s, i))?;
        on(target, "click", &splash, move |s, _| {
            select(s, i);
            activate(s);
        })?;
    }
    Ok(())
}

impl Splash {
    fn build(
        doc: &Document,
        host: &Element,
        on_start: Box<dyn FnOnce(Start)>,
    ) -> Result<Self, JsValue> {
        let root = el(doc, "div", "splash")?;
        let content = el(doc, "div", "splash-content")?;
        let title = el(doc, "h1", "splash-title")?;
        title.set_text_content(Some("ROOM 1313"));
        let menu = el(doc, "nav", "splash-menu")?;
        let whisper = el(doc, "p", "splash-whisper")?;
        whisper.set_text_content(Some(t(WHISPER)));

        let saved = store::has_game();
        let mut items = Vec::new();
        for item in Item::ALL {
            let node = el(doc, "div", "splash-item")?;
            node.set_text_content(Some(item.text()));
            if !item.enabled(saved) {
                node.class_list().add_1("disabled")?;
            }
            menu.append_child(&node)?;
            items.push(node);
        }

        content.append_child(&title)?;
        content.append_child(&menu)?;
        content.append_child(&whisper)?;
        root.append_child(&content)?;
        for class in ["splash-flash", "splash-grain"] {
            let layer = el(doc, "div", class)?;
            root.append_child(&layer)?;
        }
        host.append_child(&root)?;

        Ok(Self {
            phase: Phase::Intro,
            root,
            menu,
            items,
            whisper,
            selected: if saved { 1 } else { 0 },
            music: None,
            saved,
            on_start: Some(on_start),
        })
    }

    fn render_selection(&self) {
        for (i, item) in self.items.iter().enumerate() {
            let _ = item.class_list().toggle_with_force("selected", i == self.selected);
        }
    }

    /// Moves the selection by `dir`, wrapping and skipping disabled items.
    fn step(&mut self, dir: isize) {
        let n = Item::ALL.len() as isize;
        let mut i = self.selected as isize;
        loop {
            i = (i + dir).rem_euclid(n);
            if Item::ALL[i as usize].enabled(self.saved) {
                break;
            }
        }
        self.set_selected(i as usize);
    }

    fn set_selected(&mut self, i: usize) {
        if i == self.selected || !Item::ALL[i].enabled(self.saved) {
            return;
        }
        self.selected = i;
        self.render_selection();
        if let Some(music) = &self.music {
            music.blip();
        }
    }
}

fn begin_intro(s: &Shared) {
    {
        // The AudioContext has to be created inside the gesture handler.
        s.borrow_mut().music = Music::new().map_err(|e| web_sys::console::warn_1(&e)).ok();
    }

    // The language screen fades out over the black hold.
    let s1 = s.clone();
    after(language::FADE_MS, move || {
        if let Some(music) = &s1.borrow().music {
            music.rumble(BLACK_HOLD_MS as f64 / 1000.0);
        }
    });

    let s2 = s.clone();
    after(language::FADE_MS + BLACK_HOLD_MS, move || {
        let sp = s2.borrow();
        let _ = sp.root.class_list().add_1("lit");
        if let Some(music) = &sp.music {
            music.boom();
        }
    });

    let s3 = s.clone();
    after(language::FADE_MS + BLACK_HOLD_MS + TITLE_FLICKER_MS, move || {
        let mut sp = s3.borrow_mut();
        sp.phase = Phase::Menu;
        let _ = sp.menu.class_list().add_1("shown");
        sp.render_selection();
        if let Some(music) = &mut sp.music {
            music.start_theme();
        }
    });
}

fn on_key(s: &Shared, ev: KeyboardEvent) {
    if s.borrow().phase != Phase::Menu {
        return;
    }
    match ev.key().as_str() {
        "ArrowUp" | "w" | "W" => {
            ev.prevent_default();
            s.borrow_mut().step(-1);
        }
        "ArrowDown" | "s" | "S" => {
            ev.prevent_default();
            s.borrow_mut().step(1);
        }
        "Enter" | " " => {
            ev.prevent_default();
            activate(s);
        }
        _ => {}
    }
}

fn select(s: &Shared, i: usize) {
    let mut sp = s.borrow_mut();
    if sp.phase == Phase::Menu {
        sp.set_selected(i);
    }
}

fn activate(s: &Shared) {
    let item = {
        let sp = s.borrow();
        if sp.phase != Phase::Menu {
            return;
        }
        Item::ALL[sp.selected]
    };
    match item {
        Item::NewGame => leave(s, Start::NewGame),
        Item::Continue => leave(s, Start::Continue),
        Item::Options => crate::options::open(false),
        Item::Exit => no_exit(s),
    }
}

/// Fades title, menu and music to black, then removes the splash and starts the game, new or saved.
fn leave(s: &Shared, how: Start) {
    {
        let mut sp = s.borrow_mut();
        sp.phase = Phase::Leaving;
        let _ = sp.root.class_list().add_1("leaving");
        if let Some(music) = sp.music.take() {
            music.fade_out(LEAVE_MS as f64 / 1000.0);
        }
    }
    let s = s.clone();
    after(LEAVE_MS, move || {
        let start = {
            let mut sp = s.borrow_mut();
            sp.root.remove();
            sp.on_start.take()
        };
        if let Some(start) = start {
            start(how);
        }
    });
}

/// A browser tab can't close itself, so Exit just refuses.
fn no_exit(s: &Shared) {
    let sp = s.borrow();
    let classes = sp.whisper.class_list();
    if classes.contains("speak") {
        return;
    }
    let _ = classes.add_1("speak");
    if let Some(music) = &sp.music {
        music.sting();
    }
    let whisper = sp.whisper.clone();
    after(WHISPER_MS, move || {
        let _ = whisper.class_list().remove_1("speak");
    });
}

#[cfg(test)]
pub(crate) fn texts() -> impl Iterator<Item = &'static str> {
    Item::ALL.into_iter().map(Item::label).chain([WHISPER])
}

fn on(
    target: &EventTarget,
    ty: &str,
    s: &Shared,
    handler: impl Fn(&Shared, web_sys::Event) + 'static,
) -> Result<(), JsValue> {
    let s = s.clone();
    let cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |ev| handler(&s, ev));
    target.add_event_listener_with_callback(ty, cb.as_ref().unchecked_ref())?;
    // Lives for the page; handlers no-op once the splash is gone.
    cb.forget();
    Ok(())
}
