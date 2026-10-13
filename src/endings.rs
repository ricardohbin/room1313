//! The six endings: one tragedy for each attribute extreme in GAME.md,
//! played as cinematics with their own scores. See CINEMATICS.md.
//!
//! | Attribute   | Low (0)                  | High (100)                           |
//! |-------------|--------------------------|--------------------------------------|
//! | Reputation  | `Dismissal`              | `Assassination`                      |
//! | Sensitivity | `FatalHaunting`          | `Institutionalization`               |
//! | Arousal     | `Coma`                   | `FatalOverload`                      |

use crate::art::*;
use crate::audio::Score;
use crate::cinematic::{self, crossed, Film, Scene};
use crate::engine::{Extreme, Stat};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Ending {
    Dismissal,
    Assassination,
    FatalHaunting,
    Institutionalization,
    Coma,
    FatalOverload,
}

impl Ending {
    pub const ALL: [Ending; 6] = [
        Ending::Dismissal,
        Ending::Assassination,
        Ending::FatalHaunting,
        Ending::Institutionalization,
        Ending::Coma,
        Ending::FatalOverload,
    ];

    /// Short name, as used in the `#ending=<name>` preview link.
    pub fn name(self) -> &'static str {
        match self {
            Ending::Dismissal => "dismissal",
            Ending::Assassination => "assassination",
            Ending::FatalHaunting => "haunting",
            Ending::Institutionalization => "institution",
            Ending::Coma => "coma",
            Ending::FatalOverload => "overload",
        }
    }

    /// The ending for an attribute that hit one end of its scale.
    pub fn of(stat: Stat, extreme: Extreme) -> Self {
        match (stat, extreme) {
            (Stat::Reputation, Extreme::Low) => Ending::Dismissal,
            (Stat::Reputation, Extreme::High) => Ending::Assassination,
            (Stat::Sensitivity, Extreme::Low) => Ending::FatalHaunting,
            (Stat::Sensitivity, Extreme::High) => Ending::Institutionalization,
            (Stat::Arousal, Extreme::Low) => Ending::Coma,
            (Stat::Arousal, Extreme::High) => Ending::FatalOverload,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|e| e.name() == name)
    }

    fn film(self) -> &'static Film {
        match self {
            Ending::Dismissal => &DISMISSAL,
            Ending::Assassination => &ASSASSINATION,
            Ending::FatalHaunting => &HAUNTING,
            Ending::Institutionalization => &INSTITUTION,
            Ending::Coma => &COMA,
            Ending::FatalOverload => &OVERLOAD,
        }
    }
}

/// Plays `ending`, then calls `on_done`.
pub fn play(ending: Ending, root_id: &str, on_done: impl FnOnce() + 'static) {
    cinematic::play(ending.film(), root_id, on_done);
}

// ---------- Reputation, low ----------

static DISMISSAL: Film = Film { scenes: &DISMISSAL_SCENES, score: |m| m.score(Score::Lament) };

const DISMISSAL_SCENES: [Scene; 3] = [
    Scene::new(
        Shot::Badge,
        &["Internal Affairs took the files, the evidence, your word.", "The captain never looked up."],
        6200.0,
    )
    .cue(|m| m.rain(0.08, 700.0, 2.0))
    .sync(|m, prev, now| {
        if crossed(prev, now, BADGE_DROP) {
            m.knock(0.3);
        }
        if crossed(prev, now, GUN_DROP) {
            m.knock(0.55);
        }
    }),
    Scene::new(
        Shot::Leaving,
        &["Case 358 went to someone else. A week later, it was closed.", "Room 1313 is still for rent."],
        1500.0,
    )
    .cue(|m| m.rain(0.25, 2400.0, 1.5)),
    Scene::new(Shot::Document(Doc::Dismissal), &["Disgraced, powerless, and right all along.", "DISMISSAL"], 1600.0)
        .cue(|m| m.rain(0.05, 600.0, 1.5)),
];

// ---------- Reputation, high ----------

static ASSASSINATION: Film = Film { scenes: &ASSASSINATION_SCENES, score: |m| m.score(Score::Thrash) };

const ASSASSINATION_SCENES: [Scene; 2] = [
    Scene::new(
        Shot::DriveBy,
        &["You got too close. Someone decided you were worth a bullet.", "The motorcycle was never found."],
        5600.0,
    )
    .cue(|m| m.engine_pass(ENGINE.0, ENGINE.1))
    .sync(|m, prev, now| {
        // One shot per tick of the fire rate inside the burst.
        let shots = |t: f64| ((t.clamp(FIRE.0, FIRE.1) - FIRE.0) * FIRE_RATE).floor();
        if shots(now) > shots(prev) {
            m.gunshot();
        }
        if crossed(prev, now, SHOT_DOWN + 0.3) {
            m.knock(0.8);
        }
    }),
    Scene::new(
        Shot::Document(Doc::Assassination),
        &["The case died with you. Somebody made sure of it.", "ASSASSINATION"],
        1600.0,
    ),
];

// ---------- Sensitivity, low ----------

static HAUNTING: Film = Film { scenes: &HAUNTING_SCENES, score: |m| m.score(Score::Psychobilly) };

/// Footsteps on every stride while running.
fn footsteps(m: &mut crate::audio::Music, prev: f64, now: f64, running: bool) {
    if running && (now / STRIDE).floor() > (prev / STRIDE).floor() {
        m.knock(0.12);
    }
}

const HAUNTING_SCENES: [Scene; 3] = [
    Scene::new(
        Shot::Corridor,
        &["Footsteps all night. Doors you never opened, slamming shut.", "You couldn't see it. You could only run."],
        1200.0,
    )
    .sync(|m, prev, now| {
        footsteps(m, prev, now, corridor_running(now));
        for at in DOOR_SLAMS {
            if crossed(prev, now, at) {
                m.knock(0.9);
            }
        }
        for at in RATTLES {
            if crossed(prev, now, at) {
                m.knock(0.2);
            }
        }
    }),
    Scene::new(Shot::Stairs, &["You never saw the stairs in the dark."], 4200.0).sync(|m, prev, now| {
        footsteps(m, prev, now, now < STAIRS_HITS[0]);
        let last = STAIRS_HITS.len() - 1;
        for (i, at) in STAIRS_HITS.into_iter().enumerate() {
            if crossed(prev, now, at) {
                m.knock(if i == last { 1.0 } else { 0.5 });
            }
        }
    }),
    Scene::new(
        Shot::Document(Doc::Haunting),
        &["The report says you tripped. Nobody else was home.", "FATAL HAUNTING"],
        1600.0,
    )
    .sync(|m, prev, now| {
        if crossed(prev, now, STAMP_AT) {
            m.knock(0.6);
        }
    }),
];

// ---------- Sensitivity, high ----------

static INSTITUTION: Film = Film { scenes: &INSTITUTION_SCENES, score: |m| m.score(Score::Asylum) };

const INSTITUTION_SCENES: [Scene; 3] = [
    Scene::new(
        Shot::Orderlies,
        &["You told them about the voices. About what waits in 1313.", "They listened politely. Then they made a call."],
        1500.0,
    )
    .cue(|m| m.rain(0.15, 1800.0, 1.5))
    .sync(|m, prev, now| {
        if crossed(prev, now, VAN_SHUT) {
            m.knock(0.8);
        }
    }),
    Scene::new(Shot::Cell, &["You may have found the truth.", "No one will ever believe you."], 1500.0)
        .cue(|m| {
            m.rain(0.0, 500.0, 1.0);
            m.hum(0.03, 1.0);
        }),
    Scene::new(
        Shot::Document(Doc::Committed),
        &["The patient draws the same symbol on every wall.", "INSTITUTIONALIZATION"],
        1600.0,
    )
    .cue(|m| m.hum(0.0, 1.0))
    .sync(|m, prev, now| {
        if crossed(prev, now, STAMP_AT) {
            m.knock(0.6);
        }
    }),
];

// ---------- Arousal, low ----------

static COMA: Film = Film { scenes: &COMA_SCENES, score: |m| m.score(Score::Requiem) };

const COMA_SCENES: [Scene; 3] = [
    Scene::new(
        Shot::Collapse,
        &["Three nights without sleep. Then the floor rushed up.", "Someone shouted your name, very far away."],
        5600.0,
    )
    .sync(|m, prev, now| {
        if crossed(prev, now, COLLAPSE_FALL + 0.3) {
            m.knock(0.9);
        }
    }),
    Scene::new(Shot::Hospital, &["The machines breathe for you now.", "Case 358 waits in a drawer."], 1800.0)
        .cue(|m| m.monitor(MONITOR_BEAT, 30.0)),
    Scene::new(Shot::Document(Doc::Coma), &["Prognosis: uncertain. Visitors: none.", "COMA"], 1600.0),
];

// ---------- Arousal, high ----------

static OVERLOAD: Film = Film { scenes: &OVERLOAD_SCENES, score: |m| m.score(Score::Hardcore) };

const OVERLOAD_SCENES: [Scene; 3] = [
    Scene::new(
        Shot::Overload,
        &["The phones. The voices. The reports. Your heart raced on.", "It never slowed down again."],
        3000.0,
    )
    // The music dies with you.
    .sync(|m, prev, now| {
        if crossed(prev, now, OVERLOAD_FALL) {
            m.stop_score();
            m.knock(1.0);
        }
    }),
    Scene::new(Shot::Aftermath, &["The phone rang all weekend.", "Nobody came."], 2000.0).cue(|m| {
        m.stop_score();
        m.ring(RINGS.0, RINGS.3);
    }),
    Scene::new(
        Shot::Document(Doc::Overload),
        &["Cause of death: cardiac arrest. Age: too young.", "FATAL OVERLOAD"],
        1600.0,
    )
    .sync(|m, prev, now| {
        if crossed(prev, now, STAMP_AT) {
            m.knock(0.6);
        }
    }),
];
