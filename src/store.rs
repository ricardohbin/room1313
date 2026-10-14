//! What the browser remembers (localStorage): the saved game and the settings.
//! Storage can be missing or full (private windows), so every call is best-effort.

use crate::view::window;

const SAVE: &str = "room13.save";
const VOLUME: &str = "room13.volume";

fn storage() -> Option<web_sys::Storage> {
    window().local_storage().ok().flatten()
}

fn get(key: &str) -> Option<String> {
    storage()?.get_item(key).ok().flatten()
}

fn set(key: &str, value: &str) {
    if let Some(s) = storage() {
        let _ = s.set_item(key, value);
    }
}

fn remove(key: &str) {
    if let Some(s) = storage() {
        let _ = s.remove_item(key);
    }
}

/// The single saved game (see `Engine::save`), if any.
pub fn load_game() -> Option<String> {
    get(SAVE)
}

pub fn save_game(state: &str) {
    set(SAVE, state);
}

pub fn clear_game() {
    remove(SAVE);
}

pub fn has_game() -> bool {
    load_game().is_some()
}

/// Sound volume, 0–100.
pub fn load_volume() -> Option<u32> {
    get(VOLUME)?.parse().ok()
}

pub fn save_volume(percent: u32) {
    set(VOLUME, &percent.to_string());
}
