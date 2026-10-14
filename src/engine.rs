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

/// How a game ends: an attribute broke, or you learned the truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Loss(Stat, Extreme),
    Truth,
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
    /// Taking this choice ends the game with the truth.
    pub finale: bool,
}

impl Choice {
    pub fn new(label: &str, effect: Effect) -> Self {
        Self { label: label.into(), effect, gives: Vec::new(), takes: Vec::new(), finale: false }
    }

    pub fn finale(mut self) -> Self {
        self.finale = true;
        self
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
    /// ...and none of these.
    pub excludes: Vec<Item>,
    /// ...and only once you've held this item for this many rounds.
    pub wait: Option<(Item, u32)>,
    /// How likely it is to be drawn against the other cards (1 = normal).
    pub weight: u32,
    /// Overrides the character's `recurring` (see `once`).
    pub repeats: Option<bool>,
    pub left: Choice,
    pub right: Choice,
}

impl Scene {
    pub fn new(id: &str, who: Character, arrival: Arrival, text: &str, left: Choice, right: Choice) -> Self {
        Self {
            id: id.into(),
            who,
            arrival,
            text: text.into(),
            requires: Vec::new(),
            excludes: Vec::new(),
            wait: None,
            weight: 1,
            repeats: None,
            left,
            right,
        }
    }

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

    pub fn excludes(mut self, item: Item) -> Self {
        self.excludes.push(item);
        self
    }

    /// Holds the card back until `item` has been in your hands for `rounds` cards.
    pub fn after(mut self, item: Item, rounds: u32) -> Self {
        self.wait = Some((item, rounds));
        self
    }

    pub fn weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }

    /// A quest step: the card keeps coming back until you hold `item`
    /// (usually because you finally accepted it), whoever is asking.
    pub fn until(mut self, item: Item) -> Self {
        self.excludes.push(item);
        self.repeats = Some(true);
        self
    }

    /// Cards of recurring characters can come back; the others show only once.
    pub fn once(&self) -> bool {
        !self.repeats.unwrap_or(self.who.recurring())
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
    /// Cards played so far, and the round each item was last gained.
    round: u32,
    held_since: [u32; 32],
    finished: bool,
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
        if !scenes.iter().any(|s| !s.once() && s.requires.is_empty() && s.excludes.is_empty() && s.wait.is_none()) {
            return Err(EngineError::NoFallback);
        }
        let spent = vec![false; scenes.len()];
        Ok(Self {
            scenes,
            current,
            spent,
            stats: Stats::default(),
            inventory,
            round: 0,
            held_since: [0; 32],
            finished: false,
            rng: Rng::new(seed),
            history: Vec::new(),
        })
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

    /// Whether the game is over, and how.
    pub fn outcome(&self) -> Option<Outcome> {
        if self.finished {
            return Some(Outcome::Truth);
        }
        self.stats.extreme().map(|(stat, extreme)| Outcome::Loss(stat, extreme))
    }

    /// The attributes as they would be after choosing `side` on the current scene.
    pub fn preview(&self, side: Side) -> Stats {
        self.stats.apply(self.current().choice(side).effect)
    }

    /// Applies a choice on the current scene, then draws the next one.
    pub fn choose(&mut self, side: Side) -> &Scene {
        let choice = self.current().choice(side);
        let (stats, inventory) = (self.stats.apply(choice.effect), choice.apply(self.inventory));
        self.finished |= choice.finale;
        self.round += 1;
        for item in inventory.items().filter(|&i| !self.inventory.has(i)) {
            self.held_since[item as usize] = self.round;
        }
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
        let scene = &self.scenes[i];
        let held = |item: Item| self.inventory.has(item);
        !self.spent[i]
            && self.inventory.has_all(&scene.requires)
            && !scene.excludes.iter().any(|&item| held(item))
            && scene.wait.is_none_or(|(item, rounds)| {
                held(item) && self.round - self.held_since[item as usize] >= rounds
            })
    }

    /// A random available card (by weight), other than the one just played if there's a choice.
    /// `new` checked there's a recurring card needing nothing, so this can't come up empty.
    fn draw(&mut self) -> usize {
        let mut pool: Vec<usize> = (0..self.scenes.len()).filter(|&i| i != self.current && self.can_draw(i)).collect();
        if pool.is_empty() {
            pool.push(self.current);
        }
        let total: u32 = pool.iter().map(|&i| self.scenes[i].weight).sum();
        let mut ticket = self.rng.below(total as usize) as u32;
        for &i in &pool {
            if ticket < self.scenes[i].weight {
                return i;
            }
            ticket -= self.scenes[i].weight;
        }
        pool[0]
    }

    pub fn history(&self) -> &[(String, Side)] {
        &self.history
    }

    /// The game so far as one line of text, for `restore`: the card on the table,
    /// then the random state, round, attributes, inventory, the rounds each item
    /// was gained, the spent cards and the choices made. Card ids are plain
    /// `a-z_`, so `;` and `,` are safe separators.
    pub fn save(&self) -> String {
        let list = |items: Vec<String>| items.join(",");
        let spent = self.scenes.iter().zip(&self.spent).filter(|(_, &s)| s).map(|(s, _)| s.id.clone());
        let history = self.history.iter().map(|(id, side)| format!("{id}:{}", if *side == Side::Left { 'L' } else { 'R' }));
        [
            "1".to_string(),
            self.current().id.clone(),
            self.rng.0.to_string(),
            self.round.to_string(),
            list(Stat::ALL.iter().map(|&s| self.stats.get(s).to_string()).collect()),
            self.inventory.0.to_string(),
            list(self.held_since.iter().map(u32::to_string).collect()),
            list(spent.collect()),
            list(history.collect()),
        ]
        .join(";")
    }

    /// The game `save` described, on the given deck. `None` if the text is damaged
    /// or from a deck that has since changed (a missing card, say).
    pub fn restore(scenes: Vec<Scene>, text: &str) -> Option<Self> {
        let mut parts = text.split(';');
        if parts.next()? != "1" {
            return None;
        }
        let current = parts.next()?;
        let mut engine = Self::new(scenes, current, Inventory::default(), 0).ok()?;
        engine.rng = Rng(parts.next()?.parse().ok().filter(|&n| n != 0)?);
        engine.round = parts.next()?.parse().ok()?;
        let stats: Vec<i32> = parts.next()?.split(',').map(|n| n.parse().ok()).collect::<Option<_>>()?;
        let [r, s, a] = stats[..] else { return None };
        engine.stats = Stats::new(r, s, a);
        engine.inventory = Inventory(parts.next()?.parse().ok()?);
        let held: Vec<u32> = parts.next()?.split(',').map(|n| n.parse().ok()).collect::<Option<_>>()?;
        engine.held_since = held.try_into().ok()?;
        for id in parts.next()?.split(',').filter(|id| !id.is_empty()) {
            let i = engine.scenes.iter().position(|s| s.id == id)?;
            engine.spent[i] = true;
        }
        for entry in parts.next()?.split(',').filter(|e| !e.is_empty()) {
            let (id, side) = entry.split_once(':')?;
            let side = match side {
                "L" => Side::Left,
                "R" => Side::Right,
                _ => return None,
            };
            engine.history.push((id.to_string(), side));
        }
        Some(engine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene(id: &str, who: Character) -> Scene {
        let left = Choice::new("no", Effect::new(-30, 0, 0));
        let right = Choice::new("yes", Effect::new(0, 20, -5));
        Scene::new(id, who, Arrival::Summoned, "x", left, right)
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
    fn quest_cards_wait_and_repeat_until_accepted() {
        let mut quest = scene("quest", UNIQUE).requires(Item::EReader).after(Item::EReader, 2).until(Item::Dream);
        quest.right = quest.right.gives(Item::Dream);
        let mut start = scene("start", UNIQUE);
        start.right = start.right.gives(Item::EReader);
        let deck = vec![start, quest, scene("filler", RECURRING)];
        let mut e = Engine::new(deck, "start", Inventory::default(), 5).unwrap();
        e.choose(Side::Right); // round 1: e-reader in hand
        assert!(e.available().all(|s| s.id != "quest"), "held for 0 rounds");
        e.choose(Side::Left);
        assert!(e.available().all(|s| s.id != "quest"), "held for 1 round");
        e.choose(Side::Left);
        assert!(e.available().any(|s| s.id == "quest"), "held for 2 rounds");
        // Refusing doesn't spend it, even for a unique character; accepting retires it.
        let mut refused = 0;
        for _ in 0..50 {
            if e.current().id == "quest" {
                refused += 1;
                e.choose(Side::Left);
            } else {
                e.choose(Side::Right);
            }
        }
        assert!(refused >= 2, "the quest card kept coming back");
        while e.current().id != "quest" {
            e.choose(Side::Right);
        }
        e.choose(Side::Right);
        assert!(e.inventory().has(Item::Dream));
        assert!(e.available().all(|s| s.id != "quest"));
    }

    #[test]
    fn heavier_cards_come_up_more_often() {
        let deck = vec![scene("a", RECURRING), scene("heavy", RECURRING).weight(8), scene("b", RECURRING)];
        let mut e = Engine::new(deck, "a", Inventory::default(), 9).unwrap();
        let heavy = (0..300).filter(|_| e.choose(Side::Right).id == "heavy").count();
        // Never twice in a row, so at most half; with weight 8 against 1, close to that.
        assert!(heavy > 120, "heavy drawn {heavy} times in 300");
    }

    #[test]
    fn a_finale_choice_ends_with_the_truth() {
        let mut last = scene("last", UNIQUE);
        last.right = Choice::new("tell me", Effect::default()).finale();
        let mut e = Engine::new(vec![last, scene("filler", RECURRING)], "last", Inventory::default(), 1).unwrap();
        assert_eq!(e.outcome(), None);
        e.choose(Side::Right);
        assert_eq!(e.outcome(), Some(Outcome::Truth));
    }

    /// Plays the real deck with a sensible strategy over many shuffles: take
    /// every step of the quest when it's survivable, otherwise steer every
    /// attribute back toward 50. The truth must be reachable, and not trivially.
    #[test]
    fn the_quest_can_be_won() {
        use crate::cards::{deck, START, START_ITEMS};
        let quest = [Item::EReader, Item::Dream, Item::Fame, Item::Underworld, Item::FirstImmersion, Item::SecondImmersion, Item::Redemption];
        let (mut wins, mut rounds_to_win, mut losses) = (0, 0, 0);
        let mut deaths: Vec<String> = Vec::new();
        let games = 300;
        for seed in 0..games {
            let mut e = Engine::new(deck(), START, Inventory::of(START_ITEMS), seed).unwrap();
            for round in 0..400 {
                let pick = |side: Side| {
                    let choice = e.current().choice(side);
                    let after = e.stats().apply(choice.effect);
                    let risk: i32 = Stat::ALL.iter().map(|&s| (after.get(s) - STAT_START).abs()).sum();
                    let safe = Stat::ALL.iter().all(|&s| (12..=88).contains(&after.get(s)));
                    let progress = choice.finale || choice.gives.iter().any(|i| quest.contains(i) && !e.inventory().has(*i));
                    let dead = after.extreme().is_some();
                    (dead, !(progress && safe), risk)
                };
                let side = if pick(Side::Left) <= pick(Side::Right) { Side::Left } else { Side::Right };
                e.choose(side);
                match e.outcome() {
                    Some(Outcome::Truth) => {
                        wins += 1;
                        rounds_to_win += round + 1;
                        break;
                    }
                    Some(Outcome::Loss(stat, extreme)) => {
                        losses += 1;
                        let stage = quest.iter().rposition(|&i| e.inventory().has(i)).map_or("start", |i| quest[i].name());
                        deaths.push(format!("{}-{extreme:?} at {stage}", stat.name()));
                        break;
                    }
                    None => {}
                }
            }
        }
        let avg = if wins > 0 { rounds_to_win / wins } else { 0 };
        deaths.sort();
        let mut tally: Vec<(usize, String)> = Vec::new();
        for d in deaths {
            match tally.iter_mut().find(|(_, k)| *k == d) {
                Some((n, _)) => *n += 1,
                None => tally.push((1, d)),
            }
        }
        tally.sort_by(|a, b| b.0.cmp(&a.0));
        eprintln!("quest bot deaths: {tally:?}");
        eprintln!("quest bot: {wins}/{games} won (avg {avg} cards), {losses} lost, {} unfinished", games as usize - wins - losses);
        assert!(wins * 2 > games as usize, "a careful player should usually make it");
        assert!((25..=150).contains(&avg), "the road should take a while, but not forever");
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
    fn a_saved_game_continues_exactly_where_it_stopped() {
        use crate::cards::{deck, START, START_ITEMS};
        let mut a = Engine::new(deck(), START, Inventory::of(START_ITEMS), 11).unwrap();
        for i in 0..25 {
            a.choose(if i % 3 == 0 { Side::Left } else { Side::Right });
        }
        let mut b = Engine::restore(deck(), &a.save()).expect("restores");
        assert_eq!(b.save(), a.save());
        for _ in 0..40 {
            assert_eq!(a.choose(Side::Right).id, b.choose(Side::Right).id);
            assert_eq!((a.stats(), a.inventory()), (b.stats(), b.inventory()));
        }
        assert!(Engine::restore(deck(), "").is_none());
        assert!(Engine::restore(deck(), "1;nowhere;1;0;50,50,50;0;;;").is_none());
        assert!(Engine::restore(deck(), &a.save()[..20]).is_none());
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
