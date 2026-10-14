//! Swipe-card narrative engine (Reigns-style) compiled to WebAssembly.
//!
//! - `engine`:    pure scene graph / state machine and attributes, no browser deps.
//! - `cards`:     the cast and the deck.
//! - `swipe`:     pure drag-gesture math (offset, tilt, decision).
//! - `view`:      DOM binding via web-sys; wires pointer events to the engine.
//! - `splash`:    title screen (intro animation + menu) shown before the game.
//! - `cinematic`: player for pixel-art cutscenes with typed captions.
//! - `intro`:     the cinematic between the title screen and the first card.
//! - `endings`:   the six game-over cinematics, one per attribute extreme.
//! - `art`:       the cinematics' shots and the card portraits, painted on a canvas.
//! - `audio`:     procedural Web Audio: sound effects, themes and scores.

mod art;
mod audio;
pub mod cards;
mod cinematic;
pub mod endings;
pub mod engine;
mod intro;
mod splash;
pub mod swipe;
mod view;

use wasm_bindgen::prelude::*;

use crate::endings::Ending;
use crate::engine::{Engine, Inventory, Outcome};
use crate::view::window;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    show_version();
    // Each game draws its cards in a different order.
    let seed = (js_sys::Math::random() * u64::MAX as f64) as u64;
    // Preview an ending with `#ending=<name>` (see `Ending::name`): New Game plays it.
    // `#cards` skips the intro and deals straight away; `#card=<id>` also starts
    // on that card, with the items it needs already in hand.
    let hash = window().location().hash().unwrap_or_default();
    let preview = hash.strip_prefix("#ending=").and_then(Ending::from_name);
    let first = hash.strip_prefix("#card=");
    let deck = cards::deck();
    let (start, items) = match first.and_then(|id| deck.iter().find(|s| s.id == id)) {
        Some(scene) => (scene.id.clone(), Inventory::of(&scene.requires)),
        None => (cards::START.to_string(), Inventory::of(cards::START_ITEMS)),
    };
    let engine = Engine::new(deck, &start, items, seed).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let skip_intro = hash == "#cards" || first.is_some();
    splash::mount("app", move || match preview {
        _ if hash == "#credits" => endings::credits("app", back_to_title),
        Some(ending) => endings::play(ending, "app", back_to_title),
        None if skip_intro => deal(engine),
        None => intro::play("app", move || deal(engine)),
    })
}

/// Build version: `make` sets `ROOM_VERSION` (crate version + commit count).
const VERSION: &str = match option_env!("ROOM_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};

/// Puts the build version in a faint label in the corner of the page.
fn show_version() {
    let Some(body) = window().document().and_then(|d| d.body().map(|b| (d, b))) else { return };
    let (doc, body) = body;
    if let Ok(el) = doc.create_element("div") {
        el.set_id("version");
        el.set_text_content(Some(&format!("v{VERSION}")));
        let _ = body.append_child(&el);
    }
}

/// Starts the card game; when an attribute breaks, its ending plays,
/// and when you learn the truth, the true ending.
fn deal(engine: Engine) {
    let on_end = |outcome| match outcome {
        Outcome::Loss(stat, extreme) => endings::play(Ending::of(stat, extreme), "app", back_to_title),
        // The true ending earns the credits.
        Outcome::Truth => endings::play(Ending::Truth, "app", || endings::credits("app", back_to_title)),
    };
    if let Err(e) = view::mount(engine, "app", on_end) {
        web_sys::console::error_1(&e);
    }
}

/// After a game over, start over from the title screen.
fn back_to_title() {
    let _ = window().location().reload();
}
