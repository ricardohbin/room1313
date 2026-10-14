//! The intro cinematic, between the title screen and the first card.
//! See CINEMATICS.md.

use crate::art::{spark, Shot, LIGHTNING};
use crate::cinematic::{self, Film, Scene};

static INTRO: Film = Film { scenes: &SCENES, score: |m| m.drone() };

const SCENES: [Scene; 6] = [
    Scene::new(
        Shot::City,
        &[
            "October 13th. It had been raining for eleven days straight.",
            "That morning, a crime shocked the whole country.",
        ],
        1500.0,
    )
    .cue(|m| {
        m.rain(0.35, 2600.0, 2.0);
        for s in LIGHTNING {
            m.thunder(s + 0.6);
        }
    }),
    Scene::new(
        Shot::Hotel,
        &[
            "The Hotel Lumen. Old, huge, cheap. Nobody asks questions.",
            "Two guests checked into room 1313. Then they vanished.",
        ],
        1200.0,
    )
    .cue(|m| m.rain(0.3, 2200.0, 1.0)),
    Scene::new(
        Shot::Kindle,
        &[
            "All they left behind was a broken e-reader.",
            "Its screen would not turn off. It showed only this.",
        ],
        1800.0,
    )
    .cue(|m| {
        m.rain(0.1, 500.0, 1.5);
        m.sting();
    }),
    Scene::new(
        Shot::Room,
        &[
            "The room was torn apart. The air reeked of sulfur.",
            "Every device was fried. No light would come on.",
        ],
        1500.0,
    )
    .cue(|m| m.rain(0.14, 700.0, 1.5))
    // A crackle on the first frame of each spark burst.
    .sync(|m, prev, now| {
        if spark(now).is_some() && spark(prev).is_none() {
            m.crackle();
        }
    }),
    Scene::new(
        Shot::Cctv,
        &[
            "Hours earlier, a street camera caught the man who rented it.",
            "He kept looking back over his shoulder.",
        ],
        1000.0,
    )
    .cue(|m| {
        m.rain(0.0, 400.0, 0.3);
        m.hum(0.05, 0.3);
        m.static_burst();
    }),
    Scene::new(
        Shot::Office,
        &["The case lands on your desk. Your first.", "What happened in room 1313?"],
        5400.0,
    )
    .cut_in()
    .cue(|m| {
        m.hum(0.015, 3.0);
        m.rain(0.1, 900.0, 3.0);
    }),
];

#[cfg(test)]
pub(crate) fn texts() -> impl Iterator<Item = &'static str> {
    INTRO.lines()
}

/// Plays the intro, then calls `on_done`.
pub fn play(root_id: &str, on_done: impl FnOnce() + 'static) {
    cinematic::play(&INTRO, root_id, on_done);
}
