//! Swipe-card narrative engine (Reigns-style) compiled to WebAssembly.
//!
//! - `engine`: pure scene graph / state machine, no browser deps.
//! - `swipe`:  pure drag-gesture math (offset, tilt, decision).
//! - `view`:   DOM binding via web-sys; wires pointer events to the engine.
//! - `splash`: title screen (intro animation + menu) shown before the game.
//! - `intro`:  cinematic between the title screen and the first card.
//! - `intro_art`: the cinematic's pixel-art shots, painted on a canvas.
//! - `audio`:  procedural Web Audio for the title screen and the intro.

mod audio;
pub mod demo;
pub mod engine;
mod intro;
mod intro_art;
mod splash;
pub mod swipe;
mod view;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let engine = engine::Engine::new(demo::scenes(), demo::START)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    splash::mount("app", move || {
        intro::play("app", move || {
            if let Err(e) = view::mount(engine, "app") {
                web_sys::console::error_1(&e);
            }
        })
    })
}
