use std::fmt;

use crate::cards::{Arrival, Character, Item};

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

/// The items the detective holds: a set of `Item`s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Inventory(u32);

impl Inventory {
    pub fn of(items: &[Item]) -> Self {
        let mut inv = Self::default();
        for &item in items {
            inv.add(item);
        }
        inv
    }

    pub fn has(&self, item: Item) -> bool {
        self.0 & (1 << item as u32) != 0
    }

    pub fn has_all(&self, items: &[Item]) -> bool {
        items.iter().all(|&i| self.has(i))
    }

    pub fn add(&mut self, item: Item) {
        self.0 |= 1 << item as u32;
    }

    pub fn remove(&mut self, item: Item) {
        self.0 &= !(1 << item as u32);
    }

    pub fn items(self) -> impl Iterator<Item = Item> {
        Item::ALL.into_iter().filter(move |&i| self.has(i))
    }
}

#[derive(Debug, Clone)]
pub struct Choice {
    pub label: String,
    pub effect: Effect,
    /// Items you get, and items you lose, by making this choice.
    pub gives: Vec<Item>,
    pub takes: Vec<Item>,
}

impl Choice {
    pub fn new(label: &str, effect: Effect) -> Self {
        Self { label: label.into(), effect, gives: Vec::new(), takes: Vec::new() }
    }

    pub fn gives(mut self, item: Item) -> Self {
        self.gives.push(item);
        self
    }

    pub fn takes(mut self, item: Item) -> Self {
        self.takes.push(item);
        self
    }

    /// The inventory after this choice.
    pub fn apply(&self, mut inv: Inventory) -> Inventory {
        for &item in &self.takes {
            inv.remove(item);
        }
        for &item in &self.gives {
            inv.add(item);
        }
        inv
    }
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub id: String,
    pub who: Character,
    pub arrival: Arrival,
    pub text: String,
    /// The card can only be drawn while you hold all of these.
    pub requires: Vec<Item>,
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

    pub fn requires(mut self, item: Item) -> Self {
        self.requires.push(item);
        self
    }

    /// Cards of recurring characters can come back; the others show only once.
    pub fn once(&self) -> bool {
        !self.who.recurring()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EngineError {
    DuplicateScene(String),
    UnknownStart(String),
    /// Without a recurring card that needs no items, the deck could run dry.
    NoFallback,
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateScene(id) => write!(f, "duplicate scene id `{id}`"),
            Self::UnknownStart(id) => write!(f, "start scene `{id}` does not exist"),
            Self::NoFallback => write!(f, "the deck needs a recurring card that requires no items"),
        }
    }
}

impl std::error::Error for EngineError {}

/// Small deterministic RNG (xorshift64*), so the engine stays pure and testable.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self((seed ^ 0x9E37_79B9_7F4A_7C15) | 1)
    }

    fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        (self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 33) as usize % n
    }
}

/// Holds the deck, the card on the table, the detective's attributes and inventory.
/// After the first card, each card is drawn at random from those the inventory allows.
pub struct Engine {
    scenes: Vec<Scene>,
    current: usize,
    /// Cards that showed once and can't come back.
    spent: Vec<bool>,
    stats: Stats,
    inventory: Inventory,
    rng: Rng,
    history: Vec<(String, Side)>,
}

impl Engine {
    pub fn new(scenes: Vec<Scene>, start: &str, inventory: Inventory, seed: u64) -> Result<Self, EngineError> {
        for (i, scene) in scenes.iter().enumerate() {
            if scenes[..i].iter().any(|s| s.id == scene.id) {
                return Err(EngineError::DuplicateScene(scene.id.clone()));
            }
        }
        let current = scenes
            .iter()
            .position(|s| s.id == start)
            .ok_or_else(|| EngineError::UnknownStart(start.to_string()))?;
        if !scenes.iter().any(|s| !s.once() && s.requires.is_empty()) {
            return Err(EngineError::NoFallback);
        }
        let spent = vec![false; scenes.len()];
        Ok(Self { scenes, current, spent, stats: Stats::default(), inventory, rng: Rng::new(seed), history: Vec::new() })
    }

    pub fn current(&self) -> &Scene {
        &self.scenes[self.current]
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }

    pub fn inventory(&self) -> Inventory {
        self.inventory
    }

    /// The attributes as they would be after choosing `side` on the current scene.
    pub fn preview(&self, side: Side) -> Stats {
        self.stats.apply(self.current().choice(side).effect)
    }

    /// Applies a choice on the current scene, then draws the next one.
    pub fn choose(&mut self, side: Side) -> &Scene {
        let choice = self.current().choice(side);
        let (stats, inventory) = (self.stats.apply(choice.effect), choice.apply(self.inventory));
        (self.stats, self.inventory) = (stats, inventory);
        self.history.push((self.current().id.clone(), side));
        if self.current().once() {
            self.spent[self.current] = true;
        }
        self.current = self.draw();
        self.current()
    }

    /// Cards that could be drawn now: not spent, and their items are in hand.
    pub fn available(&self) -> impl Iterator<Item = &Scene> {
        (0..self.scenes.len()).filter(|&i| self.can_draw(i)).map(|i| &self.scenes[i])
    }

    fn can_draw(&self, i: usize) -> bool {
        !self.spent[i] && self.inventory.has_all(&self.scenes[i].requires)
    }

    /// A random available card, other than the one just played if there's a choice.
    /// `new` checked there's a recurring card needing nothing, so this can't come up empty.
    fn draw(&mut self) -> usize {
        let mut pool: Vec<usize> = (0..self.scenes.len()).filter(|&i| i != self.current && self.can_draw(i)).collect();
        if pool.is_empty() {
            pool.push(self.current);
        }
        pool[self.rng.below(pool.len())]
    }

    pub fn history(&self) -> &[(String, Side)] {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene(id: &str, who: Character) -> Scene {
        Scene {
            id: id.into(),
            who,
            arrival: Arrival::Summoned,
            text: "x".into(),
            requires: Vec::new(),
            left: Choice::new("no", Effect::new(-30, 0, 0)),
            right: Choice::new("yes", Effect::new(0, 20, -5)),
        }
    }

    const RECURRING: Character = Character::Reporter;
    const UNIQUE: Character = Character::Agent;

    #[test]
    fn choices_shift_and_clamp_attributes() {
        let mut e = Engine::new(vec![scene("a", RECURRING)], "a", Inventory::default(), 1).unwrap();
        assert_eq!(e.preview(Side::Right), Stats::new(50, 70, 45));
        assert_eq!(e.stats(), Stats::default(), "a preview doesn't commit");
        e.choose(Side::Right);
        assert_eq!(e.stats(), Stats::new(50, 70, 45));
        assert_eq!(e.stats().extreme(), None);
        e.choose(Side::Left);
        e.choose(Side::Left);
        assert_eq!(e.stats(), Stats::new(0, 70, 45));
        assert_eq!(e.stats().extreme(), Some((Stat::Reputation, Extreme::Low)));
        assert_eq!(e.history().len(), 3);
        let high = Stats::new(50, 95, 50).apply(Effect::new(0, 20, 0));
        assert_eq!(high.extreme(), Some((Stat::Sensitivity, Extreme::High)));
    }

    #[test]
    fn items_unlock_cards_and_choices_give_and_take_them() {
        let mut start = scene("start", UNIQUE);
        start.right = start.right.gives(Item::RoomKey).takes(Item::EReader);
        let deck = vec![start, scene("filler", RECURRING), scene("locked", RECURRING).requires(Item::RoomKey)];
        let mut e = Engine::new(deck, "start", Inventory::of(&[Item::EReader]), 7).unwrap();
        assert!(e.available().all(|s| s.id != "locked"));
        e.choose(Side::Right);
        assert!(e.inventory().has(Item::RoomKey));
        assert!(!e.inventory().has(Item::EReader));
        assert!(e.available().any(|s| s.id == "locked"));
    }

    #[test]
    fn unique_cards_show_once_and_draws_vary() {
        let deck = vec![scene("u1", UNIQUE), scene("u2", UNIQUE), scene("r1", RECURRING), scene("r2", RECURRING)];
        let mut e = Engine::new(deck, "u1", Inventory::default(), 42).unwrap();
        let mut seen = Vec::new();
        for _ in 0..200 {
            seen.push(e.current().id.clone());
            e.choose(Side::Right);
        }
        assert_eq!(seen.iter().filter(|id| *id == "u1").count(), 1);
        assert!(seen.iter().filter(|id| *id == "u2").count() <= 1);
        assert!(seen.windows(2).all(|w| w[0] != w[1]), "never the same card twice in a row");
        assert!(seen.iter().any(|id| id == "r1") && seen.iter().any(|id| id == "r2"));
    }

    #[test]
    fn same_seed_same_game() {
        let play = |seed| {
            let deck = vec![scene("a", RECURRING), scene("b", RECURRING), scene("c", RECURRING)];
            let mut e = Engine::new(deck, "a", Inventory::default(), seed).unwrap();
            (0..20).map(|_| e.choose(Side::Left).id.clone()).collect::<Vec<_>>()
        };
        assert_eq!(play(3), play(3));
    }

    #[test]
    fn conditions_follow_the_thresholds() {
        use Condition::*;
        let at = |vs: [i32; 6]| vs.map(Condition::of);
        assert_eq!(at([0, 1, 19, 20, 39, 40]), [Loss, Crisis, Crisis, Unease, Unease, Balanced]);
        assert_eq!(at([60, 61, 80, 81, 99, 100]), [Balanced, Unease, Unease, Crisis, Crisis, Loss]);
    }

    #[test]
    fn rejects_invalid_decks() {
        let inv = Inventory::default();
        assert_eq!(
            Engine::new(vec![scene("a", RECURRING)], "z", inv, 0).err(),
            Some(EngineError::UnknownStart("z".into()))
        );
        assert_eq!(
            Engine::new(vec![scene("a", RECURRING), scene("a", RECURRING)], "a", inv, 0).err(),
            Some(EngineError::DuplicateScene("a".into()))
        );
        let locked = scene("b", RECURRING).requires(Item::Negatives);
        assert_eq!(
            Engine::new(vec![scene("a", UNIQUE), locked], "a", inv, 0).err(),
            Some(EngineError::NoFallback)
        );
    }

    #[test]
    fn the_real_deck_is_valid_and_every_item_is_reachable() {
        use crate::cards::{deck, START, START_ITEMS};
        let deck = deck();
        assert!(Engine::new(deck.clone(), START, Inventory::of(START_ITEMS), 0).is_ok());
        for who in Character::ALL {
            assert!(deck.iter().any(|s| s.who == who), "{who:?} has no card");
        }
        for item in Item::ALL {
            let given = START_ITEMS.contains(&item)
                || deck.iter().any(|s| s.left.gives.contains(&item) || s.right.gives.contains(&item));
            assert!(given, "{item:?} can never be obtained");
        }
    }
}
