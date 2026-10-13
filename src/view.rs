//! Browser binding: builds the card DOM and the attribute HUD, handles
//! pointer drags and plays the fly-out / enter transitions between scenes.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, EventTarget, HtmlElement, PointerEvent};

use crate::art::{item_icon, stat_icon, Portrait};
use crate::cards::Item;
use crate::engine::{Choice, Condition, Engine, Extreme, Inventory, Side, Stat, Stats, STAT_START};
use crate::swipe::{Drag, SwipeConfig};

/// Must match the `.card` transition duration in style.css.
const TRANSITION_MS: i32 = 300;
const FLY_OUT_ROTATION_DEG: f64 = 30.0;
/// After the losing choice: let the gauge settle on the extreme before the ending.
const LOSS_PAUSE_MS: i32 = 1600;

type OnLoss = Box<dyn FnOnce(Stat, Extreme)>;

struct Gauge {
    root: HtmlElement,
    delta: HtmlElement,
    needle: HtmlElement,
    ghost: HtmlElement,
}

struct Ui {
    text: HtmlElement,
    card: HtmlElement,
    portrait: Portrait,
    role: HtmlElement,
    stamp: HtmlElement,
    hint: HtmlElement,
    hint_label: HtmlElement,
    hint_items: HtmlElement,
    speaker: HtmlElement,
    gauges: Vec<Gauge>,
    /// One slot per item, shown while you hold it.
    slots: Vec<(Item, HtmlElement)>,
}

struct App {
    engine: Engine,
    ui: Ui,
    swipe: SwipeConfig,
    drag: Option<Drag>,
    /// True while a choice is animating; input is ignored.
    busy: bool,
    on_loss: Option<OnLoss>,
}

type Shared = Rc<RefCell<App>>;

/// Deals the cards. When an attribute reaches either end of its scale,
/// the game stops and `on_loss` is called with it.
pub fn mount(engine: Engine, root_id: &str, on_loss: impl FnOnce(Stat, Extreme) + 'static) -> Result<(), JsValue> {
    let document = window().document().ok_or("no document")?;
    let root = document
        .get_element_by_id(root_id)
        .ok_or_else(|| format!("missing #{root_id} element"))?;

    let ui = Ui::build(&document, &root)?;
    let app = Rc::new(RefCell::new(App {
        engine,
        ui,
        swipe: SwipeConfig::default(),
        drag: None,
        busy: false,
        on_loss: Some(Box::new(on_loss)),
    }));
    {
        let app = app.borrow();
        app.render_scene();
        app.render_stats(app.engine.stats(), None);
        app.render_inventory(app.engine.inventory(), None);
    }

    let card: EventTarget = app.borrow().ui.card.clone().into();
    listen(&card, "pointerdown", &app, on_pointer_down)?;
    listen(&card, "pointermove", &app, on_pointer_move)?;
    listen(&card, "pointerup", &app, on_pointer_up)?;
    listen(&card, "pointercancel", &app, on_pointer_up)?;
    Ok(())
}

impl Gauge {
    fn build(doc: &Document, stat: Stat) -> Result<Self, JsValue> {
        let root = el(doc, "div", "gauge")?;
        let head = el(doc, "div", "gauge-head")?;
        let icon = stat_icon(doc, stat)?;
        icon.set_class_name("gauge-icon");
        let name = el(doc, "span", "gauge-name")?;
        name.set_text_content(Some(stat.name()));
        let delta = el(doc, "span", "gauge-delta")?;
        head.append_child(&icon)?;
        head.append_child(&name)?;
        head.append_child(&delta)?;

        let track = el(doc, "div", "gauge-track")?;
        let ghost = el(doc, "div", "gauge-ghost")?;
        let needle = el(doc, "div", "gauge-needle")?;
        track.append_child(&ghost)?;
        track.append_child(&needle)?;
        root.append_child(&head)?;
        root.append_child(&track)?;
        Ok(Self { root, delta, needle, ghost })
    }
}

impl Ui {
    fn build(doc: &Document, root: &web_sys::Element) -> Result<Self, JsValue> {
        let game = el(doc, "div", "game")?;
        let hud = el(doc, "div", "hud")?;
        let gauges = Stat::ALL.iter().map(|&s| Gauge::build(doc, s)).collect::<Result<Vec<_>, _>>()?;
        for g in &gauges {
            hud.append_child(&g.root)?;
        }

        let stage = el(doc, "div", "stage")?;
        let text = el(doc, "p", "scene-text")?;
        let card = el(doc, "div", "card")?;
        let (portrait, canvas) = Portrait::new(doc)?;
        canvas.set_class_name("card-portrait");
        let role = el(doc, "p", "card-role")?;
        let stamp = el(doc, "div", "card-stamp")?;
        let hint = el(doc, "div", "card-hint")?;
        let hint_label = el(doc, "p", "card-hint-label")?;
        let hint_items = el(doc, "p", "card-hint-items")?;
        hint.append_child(&hint_label)?;
        hint.append_child(&hint_items)?;
        let speaker = el(doc, "p", "speaker")?;

        let inventory = el(doc, "div", "inventory")?;
        let mut slots = Vec::with_capacity(Item::ALL.len());
        for item in Item::ALL {
            let slot = el(doc, "div", "slot")?;
            slot.set_title(item.name());
            let icon = item_icon(doc, item)?;
            icon.set_class_name("slot-icon");
            let name = el(doc, "span", "slot-name")?;
            name.set_text_content(Some(item.name()));
            slot.append_child(&icon)?;
            slot.append_child(&name)?;
            inventory.append_child(&slot)?;
            slots.push((item, slot));
        }

        card.append_child(&canvas)?;
        card.append_child(&role)?;
        card.append_child(&stamp)?;
        card.append_child(&hint)?;
        stage.append_child(&text)?;
        stage.append_child(&card)?;
        stage.append_child(&speaker)?;
        game.append_child(&hud)?;
        game.append_child(&stage)?;
        game.append_child(&inventory)?;
        let grain = el(doc, "div", "cine-grain")?;
        game.append_child(&grain)?;
        root.append_child(&game)?;

        Ok(Self { text, card, portrait, role, stamp, hint, hint_label, hint_items, speaker, gauges, slots })
    }
}

impl App {
    fn render_scene(&self) {
        let scene = self.engine.current();
        self.ui.text.set_text_content(Some(&scene.text));
        self.ui.speaker.set_text_content(Some(scene.who.name()));
        self.ui.role.set_text_content(Some(scene.who.role()));
        self.ui.stamp.set_text_content(Some(scene.arrival.label()));
        self.ui.portrait.paint(scene.who);
        self.set_hint(None);
    }

    /// Moves the needles to `stats`. With `preview`, also shows where a
    /// choice would take each attribute, faded in by how far the card leans.
    fn render_stats(&self, stats: Stats, preview: Option<(Stats, f64)>) {
        for (g, &stat) in self.ui.gauges.iter().zip(&Stat::ALL) {
            let now = stats.get(stat);
            set_style(&g.needle, "left", &format!("{now}%"));
            let classes = g.root.class_list();
            let _ = classes.remove_3("unease", "crisis", "loss");
            let condition = match Condition::of(now) {
                Condition::Balanced => None,
                Condition::Unease => Some("unease"),
                Condition::Crisis => Some("crisis"),
                Condition::Loss => Some("loss"),
            };
            if let Some(class) = condition {
                let _ = classes.add_1(class);
            }

            let change = preview.map(|(next, k)| (next.get(stat), k)).filter(|&(next, _)| next != now);
            match change {
                Some((next, strength)) => {
                    set_style(&g.ghost, "left", &format!("{next}%"));
                    set_style(&g.ghost, "opacity", &strength.to_string());
                    set_style(&g.delta, "opacity", &strength.to_string());
                    g.delta.set_inner_html(&arrows(next - now));
                    // Red when the choice pushes the attribute away from balance.
                    let worse = (next - STAT_START).abs() > (now - STAT_START).abs();
                    let _ = g.delta.class_list().toggle_with_force("worse", worse);
                }
                None => {
                    set_style(&g.ghost, "opacity", "0");
                    set_style(&g.delta, "opacity", "0");
                }
            }
        }
    }

    /// Shows the items you hold. With `preview`, marks the ones that choice would take.
    fn render_inventory(&self, inv: Inventory, preview: Option<&Choice>) {
        for (item, slot) in &self.ui.slots {
            let classes = slot.class_list();
            let _ = classes.toggle_with_force("held", inv.has(*item));
            let taken = preview.is_some_and(|c| c.takes.contains(item) && inv.has(*item));
            let _ = classes.toggle_with_force("taken", taken);
        }
    }

    fn set_hint(&self, leaning: Option<(Side, f64)>) {
        let hint = &self.ui.hint;
        let classes = hint.class_list();
        let _ = classes.remove_2("left", "right");
        match leaning {
            Some((side, strength)) => {
                let choice = self.engine.current().choice(side);
                self.ui.hint_label.set_text_content(Some(&choice.label));
                self.ui.hint_items.set_text_content(Some(&item_changes(choice, self.engine.inventory())));
                let _ = classes.add_1(side_class(side));
                set_style(hint, "opacity", &strength.to_string());
                self.render_stats(self.engine.stats(), Some((self.engine.preview(side), strength)));
                self.render_inventory(self.engine.inventory(), Some(choice));
            }
            None => {
                set_style(hint, "opacity", "0");
                self.render_stats(self.engine.stats(), None);
                self.render_inventory(self.engine.inventory(), None);
            }
        }
    }

    fn place_card(&self, dx: f64, dy: f64, rotation: f64) {
        set_style(
            &self.ui.card,
            "transform",
            &format!("translate({dx}px, {dy}px) rotate({rotation}deg)"),
        );
    }
}

/// "+ Key to 1313, - The e-reader": what a choice would actually change in your hands.
fn item_changes(choice: &Choice, inv: Inventory) -> String {
    let gained = choice.gives.iter().filter(|&&i| !inv.has(i)).map(|i| format!("+ {}", i.name()));
    let lost = choice.takes.iter().filter(|&&i| inv.has(i)).map(|i| format!("- {}", i.name()));
    gained.chain(lost).collect::<Vec<_>>().join(", ")
}

/// One arrow per step of size: small (≤5), medium (≤10), large.
fn arrows(delta: i32) -> String {
    let n = match delta.abs() {
        0 => 0,
        1..=5 => 1,
        6..=10 => 2,
        _ => 3,
    };
    let dir = if delta > 0 { "up" } else { "down" };
    format!("<i class=\"{dir}\"></i>").repeat(n)
}

fn on_pointer_down(app: &Shared, ev: PointerEvent) {
    let mut app = app.borrow_mut();
    if app.busy || app.drag.is_some() {
        return;
    }
    let _ = app.ui.card.set_pointer_capture(ev.pointer_id());
    let _ = app.ui.card.class_list().add_1("dragging");
    app.drag = Some(Drag::start(ev.client_x() as f64, ev.client_y() as f64));
}

fn on_pointer_move(app: &Shared, ev: PointerEvent) {
    let mut app = app.borrow_mut();
    let Some(drag) = app.drag.as_mut() else { return };
    drag.update(ev.client_x() as f64, ev.client_y() as f64);
    let (dx, dy) = (drag.dx(), drag.dy());

    let swipe = app.swipe;
    app.place_card(dx, dy * swipe.vertical_damping, swipe.rotation(dx));
    app.set_hint(swipe.leaning(dx));
}

fn on_pointer_up(app: &Shared, _ev: PointerEvent) {
    let decision = {
        let mut app = app.borrow_mut();
        let Some(drag) = app.drag.take() else { return };
        let _ = app.ui.card.class_list().remove_1("dragging");
        app.swipe.decide(drag.dx())
    };
    match decision {
        Some(side) => commit(app, side),
        None => {
            // Clearing the inline transform lets the CSS transition snap it back.
            let app = app.borrow();
            set_style(&app.ui.card, "transform", "");
            app.set_hint(None);
        }
    }
}

/// Applies the choice (the HUD moves right away), flies the card off-screen,
/// then brings in the next scene, or hands over to the ending.
fn commit(app: &Shared, side: Side) {
    let loss = {
        let mut a = app.borrow_mut();
        a.busy = true;
        let dir = match side {
            Side::Left => -1.0,
            Side::Right => 1.0,
        };
        let off_screen = window().inner_width().ok().and_then(|w| w.as_f64()).unwrap_or(1000.0);
        a.place_card(dir * off_screen, 0.0, dir * FLY_OUT_ROTATION_DEG);

        let (before, held) = (a.engine.stats(), a.engine.inventory());
        a.engine.choose(side);
        let (after, inv) = (a.engine.stats(), a.engine.inventory());
        set_style(&a.ui.hint, "opacity", "0");
        a.render_stats(after, None);
        a.render_inventory(inv, None);
        // Restart the pulse on the gauges that moved and the items just gained.
        for (g, &stat) in a.ui.gauges.iter().zip(&Stat::ALL) {
            if before.get(stat) != after.get(stat) {
                restart_animation(&g.root, "moved");
            }
        }
        for (item, slot) in &a.ui.slots {
            if inv.has(*item) && !held.has(*item) {
                restart_animation(slot, "new");
            }
        }
        after.extreme()
    };

    if let Some((stat, extreme)) = loss {
        let app = app.clone();
        after(LOSS_PAUSE_MS, move || {
            let on_loss = app.borrow_mut().on_loss.take();
            if let Some(on_loss) = on_loss {
                on_loss(stat, extreme);
            }
        });
        return;
    }

    let app = app.clone();
    after(TRANSITION_MS, move || {
        let mut a = app.borrow_mut();
        a.render_scene();

        // Jump (no transition) to the "entering" pose, force a reflow,
        // then drop the class so the card animates into place.
        let card = &a.ui.card;
        let _ = card.class_list().add_1("entering");
        set_style(card, "transform", "");
        let _ = card.offset_width();
        let _ = card.class_list().remove_1("entering");
        a.busy = false;
    });
}

/// Removes and re-adds `class`, so its CSS animation plays again.
fn restart_animation(node: &HtmlElement, class: &str) {
    let classes = node.class_list();
    let _ = classes.remove_1(class);
    let _ = node.offset_width();
    let _ = classes.add_1(class);
}

fn side_class(side: Side) -> &'static str {
    match side {
        Side::Left => "left",
        Side::Right => "right",
    }
}

pub(crate) fn window() -> web_sys::Window {
    web_sys::window().expect("no global window")
}

pub(crate) fn el(doc: &Document, tag: &str, class: &str) -> Result<HtmlElement, JsValue> {
    let node = doc.create_element(tag)?.dyn_into::<HtmlElement>()?;
    node.set_class_name(class);
    Ok(node)
}

fn set_style(node: &HtmlElement, prop: &str, value: &str) {
    let _ = node.style().set_property(prop, value);
}

fn listen<E: JsCast + 'static>(
    target: &EventTarget,
    ty: &str,
    app: &Shared,
    handler: fn(&Shared, E),
) -> Result<(), JsValue> {
    let app = app.clone();
    let cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |ev: web_sys::Event| {
        handler(&app, ev.unchecked_into::<E>())
    });
    target.add_event_listener_with_callback(ty, cb.as_ref().unchecked_ref())?;
    // Listeners live for the whole page lifetime.
    cb.forget();
    Ok(())
}

pub(crate) fn after(ms: i32, f: impl FnOnce() + 'static) {
    let cb = Closure::once_into_js(f);
    let _ = window().set_timeout_with_callback_and_timeout_and_arguments_0(cb.unchecked_ref(), ms);
}
