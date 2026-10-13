//! The cast and the deck: who sits across your desk, and what they ask of you.
//! Effects are (reputation, sensitivity, arousal) deltas, see GAME.md.

use crate::engine::{Choice, Effect, Scene};

pub const START: &str = "owner_key";

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

/// A choice as written in the deck: label, next card, (reputation, sensitivity, arousal).
type Pick = (&'static str, &'static str, (i32, i32, i32));

fn card(id: &str, who: Character, arrival: Arrival, text: &str, left: Pick, right: Pick) -> Scene {
    let choice = |(label, next, (r, s, a)): Pick| Choice {
        label: label.into(),
        next: next.into(),
        effect: Effect::new(r, s, a),
    };
    Scene { id: id.into(), who, arrival, text: text.into(), left: choice(left), right: choice(right) }
}

/// The first round meets everyone once; the second round then loops
/// until one of the attributes breaks.
pub fn deck() -> Vec<Scene> {
    use Arrival::*;
    use Character::*;
    vec![
        // ---------- First round ----------
        card(
            "owner_key",
            HotelOwner,
            Summoned,
            "Nobody signed the register for 1313. I gave them the key myself. It came back to the desk. Warm.",
            ("Stick to the register", "reporter_quote", (5, -10, 0)),
            ("Bag the key", "reporter_quote", (-5, 10, 5)),
        ),
        card(
            "reporter_quote",
            Reporter,
            Summoned,
            "One quote on the sulfur room, detective. Or I print what the bellboy sold me for twenty bucks.",
            ("No comment", "fanatic_number", (-10, 0, 5)),
            ("Give her a quote", "agent_device", (15, 0, 5)),
        ),
        card(
            "fanatic_number",
            Fanatic,
            Unannounced,
            "Three hundred fifty-eight! The Serpent and the Messiah share one number. Close the case!",
            ("Throw him out", "occultist_lock", (5, -10, 5)),
            ("Hear him out", "occultist_lock", (-10, 15, 5)),
        ),
        card(
            "agent_device",
            Agent,
            Unannounced,
            "Your quote made the front page. This is a matter of national interest now. The e-reader, please.",
            ("Refuse", "occultist_lock", (10, 0, 15)),
            ("Hand it over", "occultist_lock", (-10, -10, -5)),
        ),
        card(
            "occultist_lock",
            Occultist,
            Summoned,
            "That sigil is a lock, not a signature. Give me one night in the room. I'll find who holds the key.",
            ("Too dangerous", "pruitt_tapes", (0, -10, -5)),
            ("One night", "pruitt_tapes", (-10, 15, 10)),
        ),
        card(
            "pruitt_tapes",
            Conspiracist,
            Unannounced,
            "Forty hours of tapes. Thirteenth floors, black vans, blueprints. It's all connected. You want 'em?",
            ("Send him home", "owner_elevator", (5, -5, 5)),
            ("Listen to all forty", "owner_elevator", (-10, 5, -15)),
        ),
        // ---------- Second round (loops) ----------
        card(
            "owner_elevator",
            HotelOwner,
            Unannounced,
            "The elevator stops on thirteen by itself. Every night, at 3:13. Seal the floor. Please.",
            ("Seal it", "reporter_photos", (10, -10, -5)),
            ("Ride it at 3:13", "reporter_photos", (-5, 15, 15)),
        ),
        card(
            "reporter_photos",
            Reporter,
            Unannounced,
            "I developed the hotel film. Someone stands behind you in every frame. Do I run them?",
            ("Bury the photos", "fanatic_vigil", (-5, -10, 5)),
            ("Run them", "fanatic_vigil", (15, 10, 10)),
        ),
        card(
            "fanatic_vigil",
            Fanatic,
            Summoned,
            "My flock kept vigil at the Lumen all night. We prayed for the room. The room prayed back.",
            ("Break up the vigil", "agent_file", (10, -5, 10)),
            ("Join the vigil", "agent_file", (-15, 15, -10)),
        ),
        card(
            "agent_file",
            Agent,
            Summoned,
            "You asked for our file. Room 1313 has been emptied eleven times since 1913. Sign. Then forget.",
            ("Refuse to sign", "occultist_mouth", (15, 0, 10)),
            ("Sign and forget", "occultist_mouth", (-10, -10, -10)),
        ),
        card(
            "occultist_mouth",
            Occultist,
            Unannounced,
            "I spent the night in 1313. It isn't a room. It's a mouth. I can teach you to hear it.",
            ("Have him committed", "pruitt_van", (10, -15, 0)),
            ("Teach me", "pruitt_van", (-10, 20, 10)),
        ),
        card(
            "pruitt_van",
            Conspiracist,
            Summoned,
            "A black van's been outside my place since your quote ran. Hide me in a cell, I'll give you names.",
            ("Go home, Mr. Pruitt", "owner_elevator", (5, -5, -10)),
            ("Hide him in a cell", "owner_elevator", (-10, 5, 15)),
        ),
    ]
}
