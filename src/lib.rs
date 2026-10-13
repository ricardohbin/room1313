//! Swipe-card narrative engine (Reigns-style) compiled to WebAssembly.
//!
//! - `engine`:    pure scene graph / state machine, no browser deps.
//! - `swipe`:     pure drag-gesture math (offset, tilt, decision).
//! - `view`:      DOM binding via web-sys; wires pointer events to the engine.
//! - `splash`:    title screen (intro animation + menu) shown before the game.
//! - `cinematic`: player for pixel-art cutscenes with typed captions.
//! - `intro`:     the cinematic between the title screen and the first card.
//! - `endings`:   the six game-over cinematics, one per attribute extreme.
//! - `art`:       the cinematics' pixel-art shots, painted on a canvas.
//! - `audio`:     procedural Web Audio: sound effects, themes and scores.

mod art;
mod audio;
mod cinematic;
pub mod demo;
pub mod endings;
pub mod engine;
mod intro;
mod splash;
pub mod swipe;
mod view;

use wasm_bindgen::prelude::*;

use crate::endings::Ending;
use crate::view::window;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let engine = engine::Engine::new(demo::scenes(), demo::START)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    // Preview an ending with `#ending=<name>` (see `Ending::name`): New Game plays it.
    let preview = window()
        .location()
        .hash()
        .ok()
        .and_then(|h| h.strip_prefix("#ending=").and_then(Ending::from_name));
    splash::mount("app", move || match preview {
        Some(ending) => endings::play(ending, "app", back_to_title),
        None => intro::play("app", move || {
            if let Err(e) = view::mount(engine, "app") {
                web_sys::console::error_1(&e);
            }
        }),
    })
}

/// After a game over, start over from the title screen.
fn back_to_title() {
    let _ = window().location().reload();
}
