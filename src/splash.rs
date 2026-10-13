//! Title screen. Phases:
//! 1. Gate: "press any key" on black (browsers only allow audio after a gesture).
//! 2. Intro: black hold with a rising rumble, then ROOM 1313 flickers in on a boom.
//! 3. Menu: the theme loops; NEW GAME fades everything to black and hands over to the game.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, EventTarget, HtmlElement, KeyboardEvent};

use crate::audio::Music;
use crate::view::{after, el, window};

// Intro timeline. Keep in sync with the matching transitions/animations in style.css.
const GATE_FADE_MS: i32 = 800;
const BLACK_HOLD_MS: i32 = 2200;
const TITLE_FLICKER_MS: i32 = 2600;
const LEAVE_MS: i32 = 1800;
const WHISPER_MS: i32 = 2600;

#[derive(Clone, Copy, PartialEq)]
enum Item {
    NewGame,
    Continue,
    Exit,
}

impl Item {
    const ALL: [Item; 3] = [Item::NewGame, Item::Continue, Item::Exit];

    fn label(self) -> &'static str {
        match self {
            Item::NewGame => "New Game",
            Item::Continue => "Continue",
            Item::Exit => "Exit",
        }
    }

    /// Continue stays disabled until there are saves.
    fn enabled(self) -> bool {
        self != Item::Continue
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Gate,
    Intro,
    Menu,
    Leaving,
}

struct Splash {
    phase: Phase,
    root: HtmlElement,
    gate: HtmlElement,
    menu: HtmlElement,
    items: Vec<HtmlElement>,
    whisper: HtmlElement,
    selected: usize,
    music: Option<Music>,
    on_new_game: Option<Box<dyn FnOnce()>>,
}

type Shared = Rc<RefCell<Splash>>;

pub fn mount(root_id: &str, on_new_game: impl FnOnce() + 'static) -> Result<(), JsValue> {
    let document = window().document().ok_or("no document")?;
    let host = document
        .get_element_by_id(root_id)
        .ok_or_else(|| format!("missing #{root_id} element"))?;
    let splash = Rc::new(RefCell::new(Splash::build(&document, &host, Box::new(on_new_game))?));

    let root: EventTarget = splash.borrow().root.clone().into();
    on(&root, "click", &splash, |s, _| {
        if s.borrow().phase == Phase::Gate {
            begin_intro(s);
        }
    })?;
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
        on_new_game: Box<dyn FnOnce()>,
    ) -> Result<Self, JsValue> {
        let root = el(doc, "div", "splash")?;
        let content = el(doc, "div", "splash-content")?;
        let title = el(doc, "h1", "splash-title")?;
        title.set_text_content(Some("ROOM 1313"));
        let menu = el(doc, "nav", "splash-menu")?;
        let whisper = el(doc, "p", "splash-whisper")?;
        whisper.set_text_content(Some("There is no exit."));

        let mut items = Vec::new();
        for item in Item::ALL {
            let node = el(doc, "div", "splash-item")?;
            node.set_text_content(Some(item.label()));
            if !item.enabled() {
                node.class_list().add_1("disabled")?;
            }
            menu.append_child(&node)?;
            items.push(node);
        }

        let gate = el(doc, "div", "splash-gate")?;
        let prompt = el(doc, "p", "splash-prompt")?;
        prompt.set_text_content(Some("— press any key —"));
        let note = el(doc, "p", "splash-note")?;
        note.set_text_content(Some("headphones recommended"));
        gate.append_child(&prompt)?;
        gate.append_child(&note)?;

        content.append_child(&title)?;
        content.append_child(&menu)?;
        content.append_child(&whisper)?;
        root.append_child(&content)?;
        for class in ["splash-flash", "splash-grain"] {
            let layer = el(doc, "div", class)?;
            root.append_child(&layer)?;
        }
        root.append_child(&gate)?;
        host.append_child(&root)?;

        Ok(Self {
            phase: Phase::Gate,
            root,
            gate,
            menu,
            items,
            whisper,
            selected: 0,
            music: None,
            on_new_game: Some(on_new_game),
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
            if Item::ALL[i as usize].enabled() {
                break;
            }
        }
        self.set_selected(i as usize);
    }

    fn set_selected(&mut self, i: usize) {
        if i == self.selected || !Item::ALL[i].enabled() {
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
        let mut sp = s.borrow_mut();
        sp.phase = Phase::Intro;
        // The AudioContext has to be created inside the gesture handler.
        sp.music = Music::new().map_err(|e| web_sys::console::warn_1(&e)).ok();
        let _ = sp.gate.class_list().add_1("gone");
    }

    let s1 = s.clone();
    after(GATE_FADE_MS, move || {
        if let Some(music) = &s1.borrow().music {
            music.rumble(BLACK_HOLD_MS as f64 / 1000.0);
        }
    });

    let s2 = s.clone();
    after(GATE_FADE_MS + BLACK_HOLD_MS, move || {
        let sp = s2.borrow();
        let _ = sp.root.class_list().add_1("lit");
        if let Some(music) = &sp.music {
            music.boom();
        }
    });

    let s3 = s.clone();
    after(GATE_FADE_MS + BLACK_HOLD_MS + TITLE_FLICKER_MS, move || {
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
    let phase = s.borrow().phase;
    match phase {
        Phase::Gate if !ev.repeat() => begin_intro(s),
        Phase::Menu => match ev.key().as_str() {
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
        },
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
        Item::NewGame => new_game(s),
        Item::Continue => {}
        Item::Exit => no_exit(s),
    }
}

/// Fades title, menu and music to black, then removes the splash and starts the game.
fn new_game(s: &Shared) {
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
            sp.on_new_game.take()
        };
        if let Some(start) = start {
            start();
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
