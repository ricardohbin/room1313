use std::collections::HashMap;
use std::fmt;

use crate::cards::{Arrival, Character};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

/// The detective's three attributes (GAME.md), each on a 0–100 scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stat {
    Reputation,
    Sensitivity,
    Arousal,
}

impl Stat {
    pub const ALL: [Stat; 3] = [Stat::Reputation, Stat::Sensitivity, Stat::Arousal];

    pub fn name(self) -> &'static str {
        match self {
            Stat::Reputation => "Reputation",
            Stat::Sensitivity => "Sensitivity",
            Stat::Arousal => "Arousal",
        }
    }
}

/// Which end of an attribute's scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Extreme {
    Low,
    High,
}

/// How far an attribute has drifted from balance (GAME.md, deterioration thresholds).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Condition {
    Balanced,
    Unease,
    Crisis,
    Loss,
}

impl Condition {
    pub fn of(value: i32) -> Self {
        match value {
            i32::MIN..=0 | STAT_MAX.. => Condition::Loss,
            1..=19 | 81..=99 => Condition::Crisis,
            20..=39 | 61..=80 => Condition::Unease,
            _ => Condition::Balanced,
        }
    }
}

pub const STAT_MAX: i32 = 100;
pub const STAT_START: i32 = 50;

/// Values of all three attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats([i32; 3]);

impl Default for Stats {
    fn default() -> Self {
        Self([STAT_START; 3])
    }
}

impl Stats {
    pub fn new(reputation: i32, sensitivity: i32, arousal: i32) -> Self {
        Self([reputation, sensitivity, arousal])
    }

    pub fn get(&self, stat: Stat) -> i32 {
        self.0[stat as usize]
    }

    /// The attributes after `effect`, kept within 0–100.
    pub fn apply(self, effect: Effect) -> Self {
        let mut next = self;
        for stat in Stat::ALL {
            next.0[stat as usize] = (self.get(stat) + effect.get(stat)).clamp(0, STAT_MAX);
        }
        next
    }

    /// The first attribute to reach either end of its scale: that's a loss.
    pub fn extreme(&self) -> Option<(Stat, Extreme)> {
        Stat::ALL.into_iter().find_map(|stat| match self.get(stat) {
            0 => Some((stat, Extreme::Low)),
            STAT_MAX => Some((stat, Extreme::High)),
            _ => None,
        })
    }
}

/// How a choice shifts each attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Effect([i32; 3]);

impl Effect {
    pub fn new(reputation: i32, sensitivity: i32, arousal: i32) -> Self {
        Self([reputation, sensitivity, arousal])
    }

    pub fn get(&self, stat: Stat) -> i32 {
        self.0[stat as usize]
    }
}

#[derive(Debug, Clone)]
pub struct Choice {
    pub label: String,
    pub next: String,
    pub effect: Effect,
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub id: String,
    pub who: Character,
    pub arrival: Arrival,
    pub text: String,
    pub left: Choice,
    pub right: Choice,
}

impl Scene {
    pub fn choice(&self, side: Side) -> &Choice {
        match side {
            Side::Left => &self.left,
            Side::Right => &self.right,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EngineError {
    DuplicateScene(String),
    UnknownStart(String),
    DanglingChoice { scene: String, target: String },
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateScene(id) => write!(f, "duplicate scene id `{id}`"),
            Self::UnknownStart(id) => write!(f, "start scene `{id}` does not exist"),
            Self::DanglingChoice { scene, target } => {
                write!(f, "scene `{scene}` points to missing scene `{target}`")
            }
        }
    }
}

impl std::error::Error for EngineError {}

/// Holds the scene graph, the player's position in it and the detective's attributes.
/// The graph is validated up front, so navigation can never fail.
pub struct Engine {
    scenes: HashMap<String, Scene>,
    current: String,
    stats: Stats,
    history: Vec<(String, Side)>,
}

impl Engine {
    pub fn new(scenes: Vec<Scene>, start: &str) -> Result<Self, EngineError> {
        let mut map = HashMap::with_capacity(scenes.len());
        for scene in scenes {
            if map.contains_key(&scene.id) {
                return Err(EngineError::DuplicateScene(scene.id));
            }
            map.insert(scene.id.clone(), scene);
        }
        if !map.contains_key(start) {
            return Err(EngineError::UnknownStart(start.to_string()));
        }
        for scene in map.values() {
            for choice in [&scene.left, &scene.right] {
                if !map.contains_key(&choice.next) {
                    return Err(EngineError::DanglingChoice {
                        scene: scene.id.clone(),
                        target: choice.next.clone(),
                    });
                }
            }
        }
        Ok(Self {
            scenes: map,
            current: start.to_string(),
            stats: Stats::default(),
            history: Vec::new(),
        })
    }

    pub fn current(&self) -> &Scene {
        &self.scenes[&self.current]
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }

    /// The attributes as they would be after choosing `side` on the current scene.
    pub fn preview(&self, side: Side) -> Stats {
        self.stats.apply(self.current().choice(side).effect)
    }

    /// Applies a choice on the current scene and moves to the next one.
    pub fn choose(&mut self, side: Side) -> &Scene {
        self.stats = self.preview(side);
        let next = self.current().choice(side).next.clone();
        let from = std::mem::replace(&mut self.current, next);
        self.history.push((from, side));
        self.current()
    }

    pub fn history(&self) -> &[(String, Side)] {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene(id: &str, left: &str, right: &str) -> Scene {
        Scene {
            id: id.into(),
            who: Character::HotelOwner,
            arrival: Arrival::Summoned,
            text: "x".into(),
            left: Choice { label: "no".into(), next: left.into(), effect: Effect::new(-30, 0, 0) },
            right: Choice { label: "yes".into(), next: right.into(), effect: Effect::new(0, 20, -5) },
        }
    }

    #[test]
    fn navigates_and_records_history() {
        let mut e = Engine::new(vec![scene("a", "b", "c"), scene("b", "a", "a"), scene("c", "a", "a")], "a").unwrap();
        assert_eq!(e.choose(Side::Right).id, "c");
        assert_eq!(e.choose(Side::Left).id, "a");
        assert_eq!(e.history(), &[("a".into(), Side::Right), ("c".into(), Side::Left)]);
    }

    #[test]
    fn choices_shift_and_clamp_attributes() {
        let mut e = Engine::new(vec![scene("a", "a", "a")], "a").unwrap();
        assert_eq!(e.preview(Side::Right), Stats::new(50, 70, 45));
        assert_eq!(e.stats(), Stats::default(), "a preview doesn't commit");
        e.choose(Side::Right);
        assert_eq!(e.stats(), Stats::new(50, 70, 45));
        assert_eq!(e.stats().extreme(), None);
        e.choose(Side::Left);
        e.choose(Side::Left);
        assert_eq!(e.stats(), Stats::new(0, 70, 45));
        assert_eq!(e.stats().extreme(), Some((Stat::Reputation, Extreme::Low)));
        let high = Stats::new(50, 95, 50).apply(Effect::new(0, 20, 0));
        assert_eq!(high.extreme(), Some((Stat::Sensitivity, Extreme::High)));
    }

    #[test]
    fn conditions_follow_the_thresholds() {
        use Condition::*;
        let at = |vs: [i32; 6]| vs.map(Condition::of);
        assert_eq!(at([0, 1, 19, 20, 39, 40]), [Loss, Crisis, Crisis, Unease, Unease, Balanced]);
        assert_eq!(at([60, 61, 80, 81, 99, 100]), [Balanced, Unease, Unease, Crisis, Crisis, Loss]);
    }

    #[test]
    fn the_deck_is_a_valid_graph_with_everyone_in_it() {
        let deck = crate::cards::deck();
        assert!(Engine::new(deck.clone(), crate::cards::START).is_ok());
        for who in Character::ALL {
            assert!(deck.iter().any(|s| s.who == who), "{who:?} has no card");
        }
    }

    #[test]
    fn rejects_invalid_graphs() {
        assert_eq!(
            Engine::new(vec![scene("a", "a", "a")], "z").err(),
            Some(EngineError::UnknownStart("z".into()))
        );
        assert_eq!(
            Engine::new(vec![scene("a", "a", "a"), scene("a", "a", "a")], "a").err(),
            Some(EngineError::DuplicateScene("a".into()))
        );
        assert_eq!(
            Engine::new(vec![scene("a", "a", "nope")], "a").err(),
            Some(EngineError::DanglingChoice { scene: "a".into(), target: "nope".into() })
        );
    }
}
