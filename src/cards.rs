//! The cast, the items and the deck: who sits across your desk, and what they ask of you.
//! Effects are (reputation, sensitivity, arousal) deltas, see GAME.md.
//!
//! After the first card, the next one is drawn at random from the cards whose
//! required items you hold. Unique characters' cards show only once; recurring
//! characters keep coming back.

use crate::engine::{Choice, Effect, Scene};

pub const START: &str = "owner_key";
/// What's on your desk when the case lands on it.
pub const START_ITEMS: &[Item] = &[Item::EReader];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Character {
    HotelOwner,
    Reporter,
    Occultist,
    Conspiracist,
    Fanatic,
    Agent,
}

impl Character {
    pub const ALL: [Character; 6] = [
        Character::HotelOwner,
        Character::Reporter,
        Character::Occultist,
        Character::Conspiracist,
        Character::Fanatic,
        Character::Agent,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Character::HotelOwner => "Mr. Ostrowski",
            Character::Reporter => "Vera Kask",
            Character::Occultist => "Ambrose Quill",
            Character::Conspiracist => "Dale Pruitt",
            Character::Fanatic => "Brother Josiah",
            Character::Agent => "The Agent",
        }
    }

    pub fn role(self) -> &'static str {
        match self {
            Character::HotelOwner => "Owner, Hotel Lumen",
            Character::Reporter => "The Evening Star",
            Character::Occultist => "Occult investigator",
            Character::Conspiracist => "Independent researcher",
            Character::Fanatic => "Church of the Last Hour",
            Character::Agent => "Agency withheld",
        }
    }

    /// Recurring characters keep turning up; the others' cards show once each.
    pub fn recurring(self) -> bool {
        matches!(self, Character::Reporter | Character::Conspiracist | Character::Fanatic)
    }
}

/// Whether you called them in to testify, or they just showed up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arrival {
    Summoned,
    Unannounced,
}

impl Arrival {
    pub fn label(self) -> &'static str {
        match self {
            Arrival::Summoned => "Summoned",
            Arrival::Unannounced => "Unannounced",
        }
    }
}

/// Things on your desk. Holding them decides which cards can come up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    EReader,
    RoomKey,
    Negatives,
    Tapes,
    Amulet,
    AgentCard,
}

impl Item {
    pub const ALL: [Item; 6] = [Item::EReader, Item::RoomKey, Item::Negatives, Item::Tapes, Item::Amulet, Item::AgentCard];

    pub fn name(self) -> &'static str {
        match self {
            Item::EReader => "The e-reader",
            Item::RoomKey => "Key to 1313",
            Item::Negatives => "Vera's negatives",
            Item::Tapes => "Pruitt's tapes",
            Item::Amulet => "Quill's amulet",
            Item::AgentCard => "A blank card",
        }
    }
}

fn card(id: &str, who: Character, arrival: Arrival, text: &str, left: Choice, right: Choice) -> Scene {
    Scene { id: id.into(), who, arrival, text: text.into(), requires: Vec::new(), left, right }
}

/// A choice: label, then (reputation, sensitivity, arousal).
fn pick(label: &str, (r, s, a): (i32, i32, i32)) -> Choice {
    Choice::new(label, Effect::new(r, s, a))
}

pub fn deck() -> Vec<Scene> {
    use Arrival::*;
    use Character::*;
    use Item::*;
    vec![
        // ---------- The hotel owner (unique) ----------
        card(
            "owner_key",
            HotelOwner,
            Summoned,
            "Nobody signed the register for 1313. I gave them the key myself. It came back to the desk. Warm.",
            pick("Stick to the register", (5, -10, 0)),
            pick("Bag the key", (-5, 10, 5)).gives(RoomKey),
        ),
        card(
            "owner_elevator",
            HotelOwner,
            Unannounced,
            "The elevator stops on thirteen by itself. Every night, at 3:13. Give me back the key. Please.",
            pick("Give it back", (10, -10, -5)).takes(RoomKey),
            pick("Ride it at 3:13", (-5, 15, 15)),
        )
        .requires(RoomKey),
        card(
            "owner_ledger",
            HotelOwner,
            Summoned,
            "My father's ledgers. 1313 was booked every October 13th since 1913. Always the same two names.",
            pick("Not relevant", (5, -5, 0)),
            pick("Read them all night", (0, 10, -10)),
        ),
        // ---------- The occultist (unique) ----------
        card(
            "occultist_lock",
            Occultist,
            Summoned,
            "That sigil is a lock, not a signature. Give me one night in the room. Wear this until I'm back.",
            pick("Too dangerous", (0, -10, -5)),
            pick("Give him one night", (-10, 15, 10)).gives(Amulet),
        )
        .requires(EReader),
        card(
            "occultist_mouth",
            Occultist,
            Unannounced,
            "I spent the night in 1313. It isn't a room. It's a mouth. Keep the amulet on. I can teach you to hear it.",
            pick("Have him committed", (10, -15, 0)).takes(Amulet),
            pick("Teach me", (-10, 20, 10)),
        )
        .requires(Amulet),
        // ---------- The agent (unique) ----------
        card(
            "agent_device",
            Agent,
            Unannounced,
            "This is a matter of national interest now. The e-reader, please.",
            pick("Refuse", (10, 0, 15)).gives(AgentCard),
            pick("Hand it over", (-10, -10, -5)).takes(EReader),
        )
        .requires(EReader),
        card(
            "agent_file",
            Agent,
            Summoned,
            "You called the number on the card. 1313 has been emptied eleven times since 1913. Sign. Then forget.",
            pick("Refuse to sign", (15, 0, 10)),
            pick("Sign and forget", (-10, -10, -10)).takes(AgentCard),
        )
        .requires(AgentCard),
        // ---------- The reporter (recurring) ----------
        card(
            "reporter_quote",
            Reporter,
            Unannounced,
            "One quote on the sulfur room, detective. Or I print what the bellboy sold me for twenty bucks.",
            pick("No comment", (-10, 0, 5)),
            pick("Give her a quote", (15, 0, 5)),
        ),
        card(
            "reporter_photos",
            Reporter,
            Unannounced,
            "I shot a roll at the hotel. Someone stands behind you in every frame. Want the negatives?",
            pick("Not interested", (-5, -5, 5)),
            pick("Take the negatives", (5, 10, 5)).gives(Negatives),
        ),
        card(
            "reporter_darkroom",
            Reporter,
            Summoned,
            "My darkroom burned last night. Your copies are all that's left. Give them back. I'll run the story.",
            pick("Keep them", (-10, 5, 10)),
            pick("Give them back", (15, -5, 0)).takes(Negatives),
        )
        .requires(Negatives),
        // ---------- The fanatic (recurring) ----------
        card(
            "fanatic_number",
            Fanatic,
            Unannounced,
            "Three hundred fifty-eight! The Serpent and the Messiah share one number. Close the case!",
            pick("Throw him out", (5, -10, 5)),
            pick("Hear him out", (-10, 15, 5)),
        ),
        card(
            "fanatic_vigil",
            Fanatic,
            Summoned,
            "My flock kept vigil at the Lumen all night. We prayed for the room. The room prayed back.",
            pick("Break up the vigil", (10, -5, 10)),
            pick("Join the vigil", (-15, 15, -10)),
        ),
        card(
            "fanatic_negatives",
            Fanatic,
            Unannounced,
            "You have the photographs. The face behind you is HIS. Burn them, before it learns yours.",
            pick("Keep them", (5, 10, 10)),
            pick("Burn them", (-5, -10, -5)).takes(Negatives),
        )
        .requires(Negatives),
        card(
            "fanatic_reliquary",
            Fanatic,
            Unannounced,
            "That device is a reliquary of the Serpent. Let me drown it in holy water.",
            pick("Not a chance", (5, 5, 5)),
            pick("Let him take it", (-10, -15, -5)).takes(EReader),
        )
        .requires(EReader),
        // ---------- The conspiracist (recurring) ----------
        card(
            "pruitt_tapes",
            Conspiracist,
            Unannounced,
            "Forty hours of tapes. Thirteenth floors, black vans, blueprints. It's all connected. You want 'em?",
            pick("Send him home", (5, -5, 5)),
            pick("Listen to all forty", (-10, 5, -15)).gives(Tapes),
        ),
        card(
            "pruitt_van",
            Conspiracist,
            Summoned,
            "A black van's been outside my place since you took my tapes. Hide me in a cell, I'll give you names.",
            pick("Go home, Mr. Pruitt", (5, -5, -10)),
            pick("Hide him in a cell", (-10, 5, 15)),
        )
        .requires(Tapes),
        card(
            "pruitt_backwards",
            Conspiracist,
            Unannounced,
            "Tape thirteen. Play it backwards. That's a woman saying your name. YOUR name, detective!",
            pick("Erase the tape", (5, -10, -5)).takes(Tapes),
            pick("Play it again", (-5, 15, 10)),
        )
        .requires(Tapes),
        card(
            "pruitt_key",
            Conspiracist,
            Unannounced,
            "Is that the key to 1313? Lend it to me. One night. For science.",
            pick("Absolutely not", (0, 0, 5)),
            pick("Lend it to him", (-15, 5, -5)).takes(RoomKey),
        )
        .requires(RoomKey),
    ]
}
