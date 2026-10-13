//! Browser binding: builds the card DOM, handles pointer drags and
//! plays the fly-out / enter transitions between scenes.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, EventTarget, HtmlElement, PointerEvent};

use crate::engine::{Engine, Side};
use crate::swipe::{Drag, SwipeConfig};

/// Must match the `.card` transition duration in style.css.
const TRANSITION_MS: i32 = 300;
const FLY_OUT_ROTATION_DEG: f64 = 30.0;

struct Ui {
    text: HtmlElement,
    card: HtmlElement,
    portrait: HtmlElement,
    hint: HtmlElement,
    speaker: HtmlElement,
}

struct App {
    engine: Engine,
    ui: Ui,
    swipe: SwipeConfig,
    drag: Option<Drag>,
    /// True while a choice is animating; input is ignored.
    busy: bool,
}

type Shared = Rc<RefCell<App>>;

pub fn mount(engine: Engine, root_id: &str) -> Result<(), JsValue> {
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
    }));
    app.borrow().render_scene();

    let card: EventTarget = app.borrow().ui.card.clone().into();
    listen(&card, "pointerdown", &app, on_pointer_down)?;
    listen(&card, "pointermove", &app, on_pointer_move)?;
    listen(&card, "pointerup", &app, on_pointer_up)?;
    listen(&card, "pointercancel", &app, on_pointer_up)?;
    Ok(())
}

impl Ui {
    fn build(doc: &Document, root: &web_sys::Element) -> Result<Self, JsValue> {
        let stage = el(doc, "div", "stage")?;
        let text = el(doc, "p", "scene-text")?;
        let card = el(doc, "div", "card")?;
        let portrait = el(doc, "div", "card-portrait")?;
        let hint = el(doc, "div", "card-hint")?;
        let speaker = el(doc, "p", "speaker")?;

        card.append_child(&portrait)?;
        card.append_child(&hint)?;
        stage.append_child(&text)?;
        stage.append_child(&card)?;
        stage.append_child(&speaker)?;
        root.append_child(&stage)?;

        Ok(Self { text, card, portrait, hint, speaker })
    }
}

impl App {
    fn render_scene(&self) {
        let scene = self.engine.current();
        self.ui.text.set_text_content(Some(&scene.text));
        self.ui.speaker.set_text_content(Some(&scene.speaker));
        let initial = scene.speaker.chars().next().map(String::from).unwrap_or_default();
        self.ui.portrait.set_text_content(Some(&initial));
        self.set_hint(None);
    }

    fn set_hint(&self, leaning: Option<(Side, f64)>) {
        let hint = &self.ui.hint;
        let classes = hint.class_list();
        let _ = classes.remove_2("left", "right");
        match leaning {
            Some((side, strength)) => {
                let label = &self.engine.current().choice(side).label;
                hint.set_text_content(Some(label));
                let _ = classes.add_1(side_class(side));
                set_style(hint, "opacity", &strength.to_string());
            }
            None => set_style(hint, "opacity", "0"),
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

/// Flies the card off-screen, then advances the engine and brings in the next scene.
fn commit(app: &Shared, side: Side) {
    {
        let mut a = app.borrow_mut();
        a.busy = true;
        let dir = match side {
            Side::Left => -1.0,
            Side::Right => 1.0,
        };
        let off_screen = window().inner_width().ok().and_then(|w| w.as_f64()).unwrap_or(1000.0);
        a.place_card(dir * off_screen, 0.0, dir * FLY_OUT_ROTATION_DEG);
    }

    let app = app.clone();
    after(TRANSITION_MS, move || {
        let mut a = app.borrow_mut();
        a.engine.choose(side);
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

fn side_class(side: Side) -> &'static str {
    match side {
        Side::Left => "left",
        Side::Right => "right",
    }
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global window")
}

fn el(doc: &Document, tag: &str, class: &str) -> Result<HtmlElement, JsValue> {
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

fn after(ms: i32, f: impl FnOnce() + 'static) {
    let cb = Closure::once_into_js(f);
    let _ = window().set_timeout_with_callback_and_timeout_and_arguments_0(cb.unchecked_ref(), ms);
}
