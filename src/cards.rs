//! The cast, the items and the deck: who sits across your desk, and what they ask of you.
//! Effects are (reputation, sensitivity, arousal) deltas, see GAME.md.
//!
//! After the first card, the next one is drawn at random from the cards whose
//! required items you hold. Unique characters' cards show only once; recurring
//! characters keep coming back.
//!
//! The main quest (GAME.md, "The road to the truth") is a chain of items, each
//! unlocking the next character; quest cards keep coming back (`Scene::until`)
//! until you accept what they offer.

use crate::engine::{Choice, Effect, Scene};

pub const START: &str = "officer_ereader";
/// What's on your desk when the case lands on it: nothing yet. Holloway brings the e-reader.
pub const START_ITEMS: &[Item] = &[];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Character {
    Officer,
    HotelOwner,
    Reporter,
    Occultist,
    Conspiracist,
    Fanatic,
    Agent,
    Shadow,
    Prophet,
}

impl Character {
    pub const ALL: [Character; 9] = [
        Character::Officer,
        Character::HotelOwner,
        Character::Reporter,
        Character::Occultist,
        Character::Conspiracist,
        Character::Fanatic,
        Character::Agent,
        Character::Shadow,
        Character::Prophet,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Character::Officer => "Sgt. Otis Holloway",
            Character::HotelOwner => "Mr. Ostrowski",
            Character::Reporter => "Vera Kask",
            Character::Occultist => "Ambrose Quill",
            Character::Conspiracist => "Dale Pruitt",
            Character::Fanatic => "Brother Josiah",
            Character::Agent => "The Agent",
            Character::Shadow => "A shadow",
            Character::Prophet => "Mrs. Celeste Moreau",
        }
    }

    pub fn role(self) -> &'static str {
        match self {
            Character::Officer => "Your sergeant",
            Character::HotelOwner => "Owner, Hotel Lumen",
            Character::Reporter => "The Evening Star",
            Character::Occultist => "Occult investigator",
            Character::Conspiracist => "Independent researcher",
            Character::Fanatic => "Church of the Last Hour",
            Character::Agent => "Agency withheld",
            Character::Shadow => "Not on any list",
            Character::Prophet => "Prophet",
        }
    }

    /// Recurring characters keep turning up; the others' cards show once each.
    pub fn recurring(self) -> bool {
        matches!(
            self,
            Character::Officer
                | Character::Reporter
                | Character::Conspiracist
                | Character::Fanatic
                | Character::Shadow
                | Character::Prophet
        )
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
    // The main quest, in order.
    Dream,
    Fame,
    Underworld,
    FirstImmersion,
    SecondImmersion,
    Redemption,
}

impl Item {
    pub const ALL: [Item; 12] = [
        Item::EReader,
        Item::RoomKey,
        Item::Negatives,
        Item::Tapes,
        Item::Amulet,
        Item::AgentCard,
        Item::Dream,
        Item::Fame,
        Item::Underworld,
        Item::FirstImmersion,
        Item::SecondImmersion,
        Item::Redemption,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Item::EReader => "The e-reader",
            Item::RoomKey => "Key to 1313",
            Item::Negatives => "Vera's negatives",
            Item::Tapes => "Pruitt's tapes",
            Item::Amulet => "Quill's amulet",
            Item::AgentCard => "A blank card",
            Item::Dream => "A dream",
            Item::Fame => "Occult subcelebrity",
            Item::Underworld => "Access to the underworld",
            Item::FirstImmersion => "First immersion",
            Item::SecondImmersion => "Second immersion",
            Item::Redemption => "Redemption",
        }
    }
}

fn card(id: &str, who: Character, arrival: Arrival, text: &str, left: Choice, right: Choice) -> Scene {
    Scene::new(id, who, arrival, text, left, right)
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
        // ---------- Sgt. Holloway (recurring): the e-reader, then nothing but dull news ----------
        card(
            "officer_ereader",
            Officer,
            Summoned,
            "Evidence from 1313, detective. Just the e-reader. Battery's dead, screen's still on. Where do you want it?",
            pick("In my drawer", (0, 5, 0)).gives(EReader),
            pick("Switch it on", (0, 10, 5)).gives(EReader),
        ),
        card(
            "officer_lab",
            Officer,
            Summoned,
            "Lab's back. The yellow powder is sulfur. Regular sulfur. They want to know if you need that in triplicate.",
            pick("File it", (5, -5, -5)),
            pick("Triplicate", (5, -5, -10)),
        ),
        card(
            "officer_canvass",
            Officer,
            Summoned,
            "Canvassed the whole block. Nobody saw anything. Mrs. Pell says her cat's been acting funny. That's it.",
            pick("Thank Mrs. Pell", (5, -5, -5)),
            pick("Ask about the cat", (-5, 10, 0)),
        ),
        card(
            "officer_meter",
            Officer,
            Summoned,
            "Two parking tickets on the hotel van. Same night, same minute: 3:13. Probably a broken meter.",
            pick("Broken meter", (0, -5, -5)),
            pick("Pull the meter", (-5, 10, 5)),
        ),
        card(
            "officer_coffee",
            Officer,
            Unannounced,
            "Machine's broken again. I made a pot in the evidence room. Don't tell the captain. Want some?",
            pick("No, thanks", (0, 0, -5)),
            pick("Black, please", (0, 0, 10)),
        ),
        card(
            "officer_return",
            Officer,
            Unannounced,
            "Evidence sent the e-reader back. Again. Nobody signed for it. It was just... on your desk.",
            pick("Lock it away", (0, 5, 0)).gives(EReader),
            pick("Keep it close", (0, 10, 5)).gives(EReader),
        )
        .until(EReader),
        card(
            "officer_home",
            Officer,
            Unannounced,
            "Somebody keyed your car. Somebody else left flowers on it. I'll take your calls today. Go home. Sleep.",
            pick("I can't", (0, 0, 10)),
            pick("Go home", (-5, -10, -10)),
        )
        .requires(Fame),
        // ---------- The road to the truth: the shadow ----------
        card(
            "shadow_whisper",
            Shadow,
            Unannounced,
            "...you carry its eye... close yours... and I will show you the room... as it was before...",
            pick("Turn on every light", (0, -10, 5)),
            pick("Close your eyes", (0, 15, -10)).gives(Dream),
        )
        .requires(EReader)
        .after(EReader, 2)
        .until(Dream)
        .weight(3),
        // ---------- The road to the truth: on air ----------
        card(
            "reporter_podcast",
            Reporter,
            Unannounced,
            "I've started a podcast: Room 1313 After Dark. Word is you dream about the room. Come on the show.",
            pick("No comment", (-5, 0, 5)),
            pick("Go on air", (15, 10, 10)).gives(Fame),
        )
        .requires(Dream)
        .after(Dream, 4)
        .until(Fame)
        .weight(3),
        // Fame draws fire: the fanatic, Pruitt and the Agent come for you.
        card(
            "fanatic_heretic",
            Fanatic,
            Unannounced,
            "Heretic! You spoke HIS number on the radio! My flock will stand outside your house tonight. Every night.",
            pick("Arrest him", (10, 0, 15)),
            pick("Let them pray", (-10, 10, -5)),
        )
        .requires(Fame),
        card(
            "fanatic_door",
            Fanatic,
            Unannounced,
            "We painted the serpent on your door. So He passes over you. Or so He finds you. Kneel with us.",
            pick("Scrub the door", (0, -10, 10)),
            pick("Kneel", (-15, 15, 0)),
        )
        .requires(Fame),
        card(
            "pruitt_episode",
            Conspiracist,
            Unannounced,
            "Episode thirteen! You said 'mouth'! That's a code word! The van's outside YOUR place now. Let me move in!",
            pick("Get out", (5, -5, 15)),
            pick("Hear his theory", (-10, 10, 5)),
        )
        .requires(Fame),
        card(
            "pruitt_leak",
            Conspiracist,
            Unannounced,
            "I gave my listeners your home address. For your protection! Total transparency, detective!",
            pick("Arrest him", (5, 0, 15)),
            pick("Thank him", (-15, 5, 0)),
        )
        .requires(Fame),
        card(
            "agent_radio",
            Agent,
            Unannounced,
            "You talk too much on the radio. Accidents happen to people who talk too much.",
            pick("Is that a threat?", (10, 0, 20)),
            pick("Quit the show", (-15, -10, -5)),
        )
        .requires(Fame),
        card(
            "agent_tail",
            Agent,
            Unannounced,
            "We've had a car on you for six days. Tomorrow, it won't be one of ours.",
            pick("Lose the tail", (5, 0, 15)),
            pick("Ask for protection", (-15, -5, -5)),
        )
        .requires(Fame),
        // ---------- The road to the truth: below ----------
        card(
            "occultist_truth",
            Occultist,
            Unannounced,
            "The whole city heard you on the radio. So did it. You want the truth? There's a door under the city. I have the key.",
            pick("I'm a cop, Quill", (5, -10, 0)),
            pick("Show me the door", (-10, 15, 5)).gives(Underworld),
        )
        .requires(Fame)
        .after(Fame, 6)
        .until(Underworld)
        .weight(3),
        // The prophet's three immersions: each one costs dearly, and they're the only way on.
        card(
            "prophet_first",
            Prophet,
            Unannounced,
            "You've been below. I can tell. Lie down in the water with me, detective. Let it show you. It will hurt.",
            pick("Not tonight", (0, -5, 0)),
            pick("Go under", (0, 15, -15)).gives(FirstImmersion),
        )
        .requires(Underworld)
        .excludes(SecondImmersion)
        .excludes(Redemption)
        .until(FirstImmersion)
        .weight(3),
        card(
            "prophet_second",
            Prophet,
            Summoned,
            "You came back. Most don't. The second time, it learns your name. Shall we?",
            pick("Not yet", (0, -5, 0)),
            pick("Go under again", (0, 20, -15)).takes(FirstImmersion).gives(SecondImmersion),
        )
        .requires(FirstImmersion)
        .after(FirstImmersion, 3)
        .until(SecondImmersion)
        .weight(3),
        card(
            "prophet_third",
            Prophet,
            Summoned,
            "The third time, nobody comes back the same. That's the point. Are you ready to be forgiven?",
            pick("Not yet", (0, -5, 0)),
            pick("The last time", (0, 20, -20)).takes(SecondImmersion).gives(Redemption),
        )
        .requires(SecondImmersion)
        .after(SecondImmersion, 3)
        .until(Redemption)
        .weight(3),
        card(
            "shadow_truth",
            Shadow,
            Unannounced,
            "...three times you went under... and came back... do you want to know the truth?",
            pick("Not yet", (0, 5, 5)),
            pick("Tell me", (0, 0, 0)).finale(),
        )
        .requires(Redemption)
        .weight(5),
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
