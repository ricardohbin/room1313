//! Options screen: the sound volume, a way back, and (from the game) a way to the
//! main menu. It sits over whatever is on screen, the title menu or the game, and
//! swallows the keyboard while it's open. Arrows or the mouse move around, left and
//! right change the volume, Enter picks, and Escape goes back to where you were.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlElement, HtmlInputElement, KeyboardEvent};

use crate::audio::{set_volume, volume, MAX_VOLUME};
use crate::i18n::t;
use crate::view::{el, window};

/// How much left and right change the volume.
const VOLUME_STEP: u32 = 5;

pub const TITLE: &str = "Options";
const VOLUME: &str = "Volume";
const RETURN: &str = "Return to game";
const BACK: &str = "Back";
const MENU: &str = "Main menu";
const SURE: &str = "Leave for the main menu? The game is saved on every card.";
const STAY: &str = "Stay";
const LEAVE: &str = "Leave";

#[cfg(test)]
pub(crate) fn texts() -> [&'static str; 8] {
    [TITLE, VOLUME, RETURN, BACK, MENU, SURE, STAY, LEAVE]
}

thread_local! {
    static OPEN: Cell<bool> = const { Cell::new(false) };
}

#[derive(Clone, Copy, PartialEq)]
enum Entry {
    Volume,
    Close,
    Menu,
    Stay,
    Leave,
}

struct Options {
    root: HtmlElement,
    list: Vec<(Entry, HtmlElement)>,
    confirm_box: HtmlElement,
    confirm: Vec<(Entry, HtmlElement)>,
    slider: HtmlInputElement,
    readout: HtmlElement,
    confirming: bool,
    selected: usize,
    key_listener: Option<js_sys::Function>,
}

type Shared = Rc<RefCell<Options>>;

/// Shows the screen. `in_game` adds the way to the main menu and says "return to game".
pub fn open(in_game: bool) {
    if OPEN.get() {
        return;
    }
    if let Err(e) = build(in_game) {
        web_sys::console::error_1(&e);
    }
}

fn build(in_game: bool) -> Result<(), JsValue> {
    let doc = window().document().ok_or("no document")?;
    let body = doc.body().ok_or("no body")?;

    let root = el(&doc, "div", "options")?;
    let panel = el(&doc, "div", "options-panel")?;
    let title = el(&doc, "h2", "options-title")?;
    title.set_text_content(Some(t(TITLE)));

    let menu = el(&doc, "nav", "options-menu")?;
    let mut list = Vec::new();

    let volume_row = el(&doc, "div", "splash-item options-item options-volume")?;
    let label = el(&doc, "span", "")?;
    label.set_text_content(Some(t(VOLUME)));
    let slider = doc.create_element("input")?.dyn_into::<HtmlInputElement>()?;
    slider.set_type("range");
    slider.set_min("0");
    slider.set_max(&MAX_VOLUME.to_string());
    slider.set_value(&volume().to_string());
    slider.set_tab_index(-1);
    let readout = el(&doc, "span", "options-readout")?;
    volume_row.append_child(&label)?;
    volume_row.append_child(&slider)?;
    volume_row.append_child(&readout)?;
    menu.append_child(&volume_row)?;
    list.push((Entry::Volume, volume_row));

    let item = |entries: &mut Vec<(Entry, HtmlElement)>, parent: &HtmlElement, entry, text: &str| {
        let node = el(&doc, "div", "splash-item options-item")?;
        node.set_text_content(Some(t(text)));
        parent.append_child(&node)?;
        entries.push((entry, node));
        Ok::<_, JsValue>(())
    };
    item(&mut list, &menu, Entry::Close, if in_game { RETURN } else { BACK })?;
    if in_game {
        item(&mut list, &menu, Entry::Menu, MENU)?;
    }

    let confirm_box = el(&doc, "nav", "options-menu options-confirm")?;
    let question = el(&doc, "p", "options-question")?;
    question.set_text_content(Some(t(SURE)));
    confirm_box.append_child(&question)?;
    let mut confirm = Vec::new();
    item(&mut confirm, &confirm_box, Entry::Stay, STAY)?;
    item(&mut confirm, &confirm_box, Entry::Leave, LEAVE)?;

    panel.append_child(&title)?;
    panel.append_child(&menu)?;
    panel.append_child(&confirm_box)?;
    root.append_child(&panel)?;
    body.append_child(&root)?;

    let options = Rc::new(RefCell::new(Options {
        root,
        list,
        confirm_box,
        confirm,
        slider,
        readout,
        confirming: false,
        selected: 0,
        key_listener: None,
    }));
    {
        let mut o = options.borrow_mut();
        o.render();
        // Capture on the window runs before the menu's or the game's own key handlers.
        let cb = Closure::<dyn FnMut(web_sys::Event)>::new({
            let options = options.clone();
            move |ev: web_sys::Event| on_key(&options, ev.unchecked_into())
        });
        let f: js_sys::Function = cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
        window().add_event_listener_with_callback_and_bool("keydown", &f, true)?;
        // One small closure leaks per visit; it can't be dropped while it may still be running.
        cb.forget();
        o.key_listener = Some(f);
    }
    OPEN.set(true);

    let entries: Vec<(Entry, HtmlElement)> = {
        let o = options.borrow();
        o.list.iter().chain(&o.confirm).cloned().collect()
    };
    for (entry, node) in entries {
        let target: &EventTarget = &node;
        on(target, "pointerenter", &options, move |o, _| o.borrow_mut().hover(entry))?;
        on(target, "click", &options, move |o, _| {
            o.borrow_mut().hover(entry);
            activate(o, entry);
        })?;
    }
    let slider: EventTarget = options.borrow().slider.clone().into();
    on(&slider, "input", &options, |o, _| {
        let value = o.borrow().slider.value().parse().unwrap_or(0);
        change_volume(o, value);
    })?;
    Ok(())
}

impl Options {
    fn entries(&self) -> &[(Entry, HtmlElement)] {
        if self.confirming {
            &self.confirm
        } else {
            &self.list
        }
    }

    fn render(&self) {
        let _ = self.confirm_box.style().set_property("display", if self.confirming { "flex" } else { "none" });
        let _ = self.list[0].1.parent_element().map(|menu| {
            let menu: HtmlElement = menu.unchecked_into();
            menu.style().set_property("display", if self.confirming { "none" } else { "flex" })
        });
        for (i, (_, node)) in self.entries().iter().enumerate() {
            let _ = node.class_list().toggle_with_force("selected", i == self.selected);
        }
        self.slider.set_value(&volume().to_string());
        self.readout.set_text_content(Some(&format!("{}%", volume())));
    }

    fn hover(&mut self, entry: Entry) {
        if let Some(i) = self.entries().iter().position(|(e, _)| *e == entry) {
            self.selected = i;
            self.render();
        }
    }

    fn step(&mut self, dir: isize) {
        let n = self.entries().len() as isize;
        self.selected = (self.selected as isize + dir).rem_euclid(n) as usize;
        self.render();
    }

    fn set_confirming(&mut self, confirming: bool) {
        self.confirming = confirming;
        // On the question, "Stay" is the one under the cursor.
        self.selected = 0;
        self.render();
    }
}

fn on_key(o: &Shared, ev: KeyboardEvent) {
    if ev.ctrl_key() || ev.alt_key() || ev.meta_key() {
        return;
    }
    // Nothing reaches the screen underneath while this one is up.
    ev.stop_immediate_propagation();
    ev.prevent_default();
    let (selected, confirming) = {
        let o = o.borrow();
        (o.entries()[o.selected].0, o.confirming)
    };
    match ev.key().as_str() {
        "Escape" if confirming => o.borrow_mut().set_confirming(false),
        "Escape" => close(o),
        "ArrowUp" | "w" | "W" => o.borrow_mut().step(-1),
        "ArrowDown" | "s" | "S" => o.borrow_mut().step(1),
        "ArrowLeft" | "a" | "A" if selected == Entry::Volume => {
            change_volume(o, volume().saturating_sub(VOLUME_STEP));
        }
        "ArrowRight" | "d" | "D" if selected == Entry::Volume => change_volume(o, volume() + VOLUME_STEP),
        "Enter" | " " if !ev.repeat() => activate(o, selected),
        _ => {}
    }
}

fn change_volume(o: &Shared, percent: u32) {
    set_volume(percent);
    o.borrow().render();
}

fn activate(o: &Shared, entry: Entry) {
    match entry {
        Entry::Volume => {}
        Entry::Stay => o.borrow_mut().set_confirming(false),
        Entry::Close => close(o),
        Entry::Menu => o.borrow_mut().set_confirming(true),
        // Starting over from the title screen is a fresh page load (the game is already saved).
        Entry::Leave => {
            let _ = window().location().reload();
        }
    }
}

fn close(o: &Shared) {
    let mut o = o.borrow_mut();
    if let Some(f) = o.key_listener.take() {
        let _ = window().remove_event_listener_with_callback_and_bool("keydown", &f, true);
    }
    o.root.remove();
    OPEN.set(false);
}

fn on(
    target: &EventTarget,
    ty: &str,
    o: &Shared,
    handler: impl Fn(&Shared, web_sys::Event) + 'static,
) -> Result<(), JsValue> {
    let o = o.clone();
    let cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |ev| handler(&o, ev));
    target.add_event_listener_with_callback(ty, cb.as_ref().unchecked_ref())?;
    // Lives as long as the element it's on (a few small closures leak per visit).
    cb.forget();
    Ok(())
}
