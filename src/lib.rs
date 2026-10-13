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
use crate::engine::Engine;
use crate::view::window;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let engine = Engine::new(cards::deck(), cards::START).map_err(|e| JsValue::from_str(&e.to_string()))?;
    // Preview an ending with `#ending=<name>` (see `Ending::name`): New Game plays it.
    // `#cards` skips the intro and deals straight away.
    let hash = window().location().hash().unwrap_or_default();
    let preview = hash.strip_prefix("#ending=").and_then(Ending::from_name);
    let skip_intro = hash == "#cards";
    splash::mount("app", move || match preview {
        Some(ending) => endings::play(ending, "app", back_to_title),
        None if skip_intro => deal(engine),
        None => intro::play("app", move || deal(engine)),
    })
}

/// Starts the card game; when an attribute breaks, its ending plays.
fn deal(engine: Engine) {
    let on_loss = |stat, extreme| endings::play(Ending::of(stat, extreme), "app", back_to_title);
    if let Err(e) = view::mount(engine, "app", on_loss) {
        web_sys::console::error_1(&e);
    }
}

/// After a game over, start over from the title screen.
fn back_to_title() {
    let _ = window().location().reload();
}
