//! The player's sound volume: remembered between visits and applied to the
//! master gain of every `Music`, including the one already playing.

use std::cell::{Cell, RefCell};

use web_sys::GainNode;

use crate::store;

/// Master gain at full volume.
const FULL: f32 = 0.8;
pub const MAX: u32 = 100;

thread_local! {
    static VOLUME: Cell<Option<u32>> = const { Cell::new(None) };
    /// The master gain of the latest `Music`, so the slider is heard right away.
    static MASTER: RefCell<Option<GainNode>> = const { RefCell::new(None) };
}

/// Volume in percent (100 until the player changes it).
pub fn volume() -> u32 {
    match VOLUME.get() {
        Some(v) => v,
        None => {
            let v = store::load_volume().map_or(MAX, |v| v.min(MAX));
            VOLUME.set(Some(v));
            v
        }
    }
}

pub fn set_volume(percent: u32) {
    let percent = percent.min(MAX);
    VOLUME.set(Some(percent));
    store::save_volume(percent);
    MASTER.with_borrow(|master| {
        if let Some(master) = master {
            master.gain().set_value(level());
        }
    });
}

fn level() -> f32 {
    FULL * volume() as f32 / MAX as f32
}

/// Sets `master` to the player's volume and keeps it in step with the slider.
pub(super) fn track(master: &GainNode) {
    master.gain().set_value(level());
    MASTER.set(Some(master.clone()));
}
