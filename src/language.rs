//! The first screen: pick a language, by its own name and its country's flag.
//! The prompt and the note speak the language under the cursor. Picking one is
//! also the gesture that lets the title screen start its music.
//! The choice is remembered and preselected next time.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlElement, KeyboardEvent};

use crate::art::flag;
use crate::i18n::{self, Lang};
use crate::view::{after, el, window};

/// The screen fades out over this long. Keep in sync with `.language` in style.css.
pub const FADE_MS: i32 = 800;
const STORAGE_KEY: &str = "room1313.lang";
const PROMPT: &str = "Choose your language";
const NOTE: &str = "headphones recommended";

struct Chooser {
    root: HtmlElement,
    prompt: HtmlElement,
    note: HtmlElement,
    items: Vec<HtmlElement>,
    selected: usize,
    on_pick: Option<Box<dyn FnOnce()>>,
}

type Shared = Rc<RefCell<Chooser>>;

/// Shows the language screen; once a language is picked, it's set for the
/// whole game and `on_pick` runs (still inside the click or key handler).
pub fn choose(root_id: &str, on_pick: impl FnOnce() + 'static) -> Result<(), JsValue> {
    let doc = window().document().ok_or("no document")?;
    let host = doc
        .get_element_by_id(root_id)
        .ok_or_else(|| format!("missing #{root_id} element"))?;

    let root = el(&doc, "div", "language")?;
    let prompt = el(&doc, "p", "language-prompt")?;
    let menu = el(&doc, "nav", "language-menu")?;
    let note = el(&doc, "p", "language-note")?;
    let mut items = Vec::new();
    for lang in Lang::ALL {
        let item = el(&doc, "div", "language-item")?;
        item.set_attribute("lang", lang.code())?;
        let canvas = flag(&doc, lang)?;
        canvas.set_class_name("language-flag");
        let name = el(&doc, "span", "language-name")?;
        name.set_text_content(Some(lang.native_name()));
        let country = el(&doc, "span", "language-country")?;
        country.set_text_content(Some(lang.country()));
        item.append_child(&canvas)?;
        item.append_child(&name)?;
        item.append_child(&country)?;
        menu.append_child(&item)?;
        items.push(item);
    }
    root.append_child(&prompt)?;
    root.append_child(&menu)?;
    root.append_child(&note)?;
    let grain = el(&doc, "div", "splash-grain")?;
    root.append_child(&grain)?;
    host.append_child(&root)?;

    let selected = Lang::ALL.iter().position(|&l| l == preferred()).unwrap_or(0);
    let chooser = Rc::new(RefCell::new(Chooser {
        root,
        prompt,
        note,
        items: items.clone(),
        selected,
        on_pick: Some(Box::new(on_pick)),
    }));
    chooser.borrow().render();

    for (i, item) in items.iter().enumerate() {
        let target: &EventTarget = item;
        on(target, "pointerenter", &chooser, move |c, _| c.borrow_mut().select(i))?;
        on(target, "click", &chooser, move |c, _| {
            c.borrow_mut().select(i);
            pick(c);
        })?;
    }
    on(&doc.into(), "keydown", &chooser, |c, ev| {
        let ev: KeyboardEvent = ev.unchecked_into();
        let n = Lang::ALL.len();
        let step = |c: &Shared, d: usize| {
            let i = (c.borrow().selected + d) % n;
            c.borrow_mut().select(i);
        };
        match ev.key().as_str() {
            "ArrowUp" | "ArrowLeft" | "w" | "W" => step(c, n - 1),
            "ArrowDown" | "ArrowRight" | "s" | "S" => step(c, 1),
            "Enter" | " " if !ev.repeat() => pick(c),
            _ => return,
        }
        ev.prevent_default();
    })?;
    Ok(())
}

impl Chooser {
    fn lang(&self) -> Lang {
        Lang::ALL[self.selected]
    }

    fn select(&mut self, i: usize) {
        if self.on_pick.is_some() && i != self.selected {
            self.selected = i;
            self.render();
        }
    }

    fn render(&self) {
        for (i, item) in self.items.iter().enumerate() {
            let _ = item.class_list().toggle_with_force("selected", i == self.selected);
        }
        let lang = self.lang();
        self.prompt.set_text_content(Some(lang.tr(PROMPT)));
        self.note.set_text_content(Some(lang.tr(NOTE)));
    }
}

/// Sets the language, hands over, and fades the screen out.
fn pick(c: &Shared) {
    let (lang, on_pick) = {
        let mut ch = c.borrow_mut();
        (ch.lang(), ch.on_pick.take())
    };
    let Some(on_pick) = on_pick else { return };
    i18n::set(lang);
    remember(lang);
    if let Some(html) = window().document().and_then(|d| d.document_element()) {
        let _ = html.set_attribute("lang", lang.code());
    }
    let root = c.borrow().root.clone();
    let _ = root.class_list().add_1("gone");
    on_pick();
    after(FADE_MS, move || root.remove());
}

/// Last time's choice, else the browser's language, else English.
fn preferred() -> Lang {
    let w = window();
    let saved = w.local_storage().ok().flatten().and_then(|s| s.get_item(STORAGE_KEY).ok().flatten());
    saved
        .or_else(|| w.navigator().language())
        .and_then(|code| Lang::from_code(&code))
        .unwrap_or(Lang::EnUs)
}

fn remember(lang: Lang) {
    if let Ok(Some(storage)) = window().local_storage() {
        let _ = storage.set_item(STORAGE_KEY, lang.code());
    }
}

#[cfg(test)]
pub(crate) fn texts() -> [&'static str; 2] {
    [PROMPT, NOTE]
}

fn on(
    target: &EventTarget,
    ty: &str,
    c: &Shared,
    handler: impl Fn(&Shared, web_sys::Event) + 'static,
) -> Result<(), JsValue> {
    let c = c.clone();
    let cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |ev| {
        // Handlers outlive the screen; ignore input once a language is picked.
        if c.borrow().on_pick.is_some() {
            handler(&c, ev);
        }
    });
    target.add_event_listener_with_callback(ty, cb.as_ref().unchecked_ref())?;
    cb.forget();
    Ok(())
}
