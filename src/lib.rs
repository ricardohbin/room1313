//! Swipe-card narrative engine (Reigns-style) compiled to WebAssembly.
//!
//! - `engine`: pure scene graph / state machine, no browser deps.
//! - `swipe`:  pure drag-gesture math (offset, tilt, decision).
//! - `view`:   DOM binding via web-sys; wires pointer events to the engine.

pub mod demo;
pub mod engine;
pub mod swipe;
mod view;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let engine = engine::Engine::new(demo::scenes(), demo::START)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    view::mount(engine, "app")
}
