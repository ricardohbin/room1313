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
    // Side items: no use to the quest, but people have opinions about them.
    Cat,
    Flask,
    Recorder,
    Pamphlets,
    Plaque,
    Voucher,
    Invoice,
}

impl Item {
    pub const ALL: [Item; 19] = [
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
        Item::Cat,
        Item::Flask,
        Item::Recorder,
        Item::Pamphlets,
        Item::Plaque,
        Item::Voucher,
        Item::Invoice,
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
            Item::Cat => "Mrs. Pell's cat",
            Item::Flask => "A dead man's flask",
            Item::Recorder => "Vera's recorder",
            Item::Pamphlets => "The Last Hour pamphlets",
            Item::Plaque => "Employee of the Month",
            Item::Voucher => "A free night in 1313",
            Item::Invoice => "Quill's invoice",
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
        // ================= Fillers =================
        // Nothing here moves the quest; it's the city getting on with being itself.

        // ---------- Holloway: useless clues, delivered too closely ----------
        card(
            "officer_hair",
            Officer,
            Summoned,
            "Found a hair in the drain of 1313. Long, grey. Lab says it's from a wig. A cheap wig. Want to smell it?",
            pick("Absolutely not", (0, -5, 5)),
            pick("...Fine", (-5, 10, -5)),
        ),
        card(
            "officer_lunch",
            Officer,
            Summoned,
            "I ate lunch at the crime scene. Egg salad. Forensics thinks some of the sulfur smell might've been me.",
            pick("Write him up", (5, -5, 5)),
            pick("Let it go", (-5, 0, -5)),
        ),
        card(
            "officer_toenail",
            Officer,
            Summoned,
            "Toenail clipping, under the bed in 1313. I bagged it, labeled it. Then I lost the label. Big toe, I think.",
            pick("Find the label", (5, 0, 5)),
            pick("Burn the bag", (-5, -5, -5)),
        ),
        card(
            "officer_shoes",
            Officer,
            Unannounced,
            "Took my shoes off in the evidence room. Long shift. The e-reader's screen went dark for a second. Coincidence?",
            pick("Shoes on. Now.", (0, -5, 5)),
            pick("Do it again", (-5, 10, 0)),
        )
        .requires(EReader),
        card(
            "officer_cat",
            Officer,
            Summoned,
            "Mrs. Pell left her cat with us. Says it won't stop staring at your desk. So I guess it's yours now.",
            pick("Take it to the pound", (5, -5, 0)),
            pick("Keep the cat", (0, 5, -5)).gives(Cat),
        )
        .excludes(Cat),
        card(
            "officer_flask",
            Officer,
            Unannounced,
            "My uncle's flask. Rye. For the nerves. He carried it thirty years on the force. Then one night he didn't.",
            pick("Not on duty", (5, 0, 5)),
            pick("Just a sip", (-5, 5, -10)).gives(Flask),
        )
        .excludes(Flask),
        card(
            "officer_flask_back",
            Officer,
            Summoned,
            "Could I have the flask back? It's all I have of him. It's empty, isn't it. It's okay. It's okay.",
            pick("Keep it", (-5, 0, 5)),
            pick("Give it back", (5, 0, -5)).takes(Flask),
        )
        .requires(Flask)
        .after(Flask, 5),
        card(
            "officer_cat_sick",
            Officer,
            Unannounced,
            "Your cat ate my sandwich. Then threw it up on your files. In the shape of a three. And two ones. I cleaned it.",
            pick("Feed the cat", (0, 0, -5)),
            pick("Study the stain", (-5, 10, 5)),
        )
        .requires(Cat),
        // ---------- Ostrowski: you're bad for business ----------
        card(
            "owner_tourists",
            HotelOwner,
            Unannounced,
            "Since your face is in the papers, tourists ask for 1313 by name. They photograph the carpet. Make it stop.",
            pick("Not my problem", (-10, 0, 0)),
            pick("Post an officer", (5, 0, 10)),
        ),
        card(
            "owner_elevator_sign",
            HotelOwner,
            Unannounced,
            "I took the 13 off the elevator. Guests use the stairs now. They count the floors aloud. Some count fourteen.",
            pick("Put it back", (5, -5, 5)),
            pick("Count them yourself", (-5, 10, 5)),
        ),
        card(
            "owner_lawyer",
            HotelOwner,
            Summoned,
            "My lawyer calls your investigation a 'reputational tort'. I don't know what that is. I hate it all the same.",
            pick("Call his bluff", (10, 0, 10)),
            pick("Promise discretion", (-5, 0, -5)),
        ),
        card(
            "owner_flask",
            HotelOwner,
            Unannounced,
            "You smell of rye. In MY lobby. In front of the PRESS. Do you know what they'll write?",
            pick("Hide the flask", (5, 0, 5)),
            pick("Offer him a sip", (-10, 5, -5)),
        )
        .requires(Flask),
        // ---------- Vera: this case is her ticket ----------
        card(
            "reporter_hero",
            Reporter,
            Unannounced,
            "Front page: haunted room, heroic cop. That's you. In return you call me first. Always. Deal?",
            pick("Find another hero", (-5, 0, 0)),
            pick("Deal", (15, 0, 10)),
        ),
        card(
            "reporter_recorder",
            Reporter,
            Unannounced,
            "Oops, I left my recorder on your desk. Don't touch it. It's... charging.",
            pick("Throw it out", (0, 0, 5)),
            pick("Leave it be", (5, 0, -5)).gives(Recorder),
        )
        .excludes(Recorder),
        card(
            "reporter_book",
            Reporter,
            Summoned,
            "A publisher wants my book on 1313. I've written the ending: you die. Unless you give me a better one.",
            pick("Let me die in print", (-10, 0, -5)),
            pick("Give her a quote", (10, 5, 5)),
        ),
        card(
            "reporter_cat",
            Reporter,
            Unannounced,
            "A shot of you with the cat. 'The Detective and the Cat That Saw Too Much.' Readers will die.",
            pick("No pictures", (-5, 0, 0)),
            pick("Smile", (10, 5, 5)),
        )
        .requires(Cat),
        card(
            "reporter_tape_voice",
            Reporter,
            Summoned,
            "My recorder caught something at 3:13 last night. A voice, saying your name. Can I have it back? Please?",
            pick("Keep it", (-5, 10, 5)),
            pick("Give it back", (5, -5, -5)).takes(Recorder),
        )
        .requires(Recorder)
        .after(Recorder, 3),
        // ---------- Quill: kind, and keeping score ----------
        card(
            "occultist_tea",
            Occultist,
            Unannounced,
            "Chamomile, valerian, a pinch of ash from the room. For your sleep. Also, sign this permit to dig up your basement.",
            pick("No permits", (0, 5, 5)),
            pick("Drink, and sign", (-10, 5, -10)),
        ),
        card(
            "occultist_borrow_cat",
            Occultist,
            Unannounced,
            "What a magnificent cat. A natural medium. May I borrow her? Only for a night. Or a decade.",
            pick("She stays", (0, -5, 5)),
            pick("Lend him the cat", (-5, 10, -5)).takes(Cat),
        )
        .requires(Cat),
        card(
            "occultist_book",
            Occultist,
            Summoned,
            "I'm writing the definitive work on 1313. You'll be in the acknowledgments. Small print. Tell me everything.",
            pick("Not a word", (0, -5, 5)),
            pick("Everything", (-5, 10, 0)),
        ),
        card(
            "occultist_honest",
            Occultist,
            Unannounced,
            "I'll be honest. I like you. You're useful. In that order. Stop going to the hotel alone.",
            pick("I go where I want", (5, 5, 10)),
            pick("Take him along", (-5, 5, -5)),
        ),
        // ---------- Pruitt: it's all connected ----------
        card(
            "pruitt_pigeons",
            Conspiracist,
            Unannounced,
            "Pigeons aren't real, detective. Drones. And every one of them flies over the Lumen at 3:13. Check my math!",
            pick("Get out", (5, -5, 5)),
            pick("Check his math", (-10, 5, -5)),
        ),
        card(
            "pruitt_hat",
            Conspiracist,
            Unannounced,
            "I made you a hat. Aluminum, triple-layered. Blocks the room's signal. Fifty bucks. Friend price.",
            pick("No hat", (0, -5, 5)),
            pick("Wear the hat", (-15, -10, -5)),
        ),
        card(
            "pruitt_recorder",
            Conspiracist,
            Unannounced,
            "That's a RECORDER! Who's recording you?! THEM! Give it here, I'll find out who!",
            pick("It's Vera's", (0, 0, 5)),
            pick("Give it to him", (-5, 5, -5)).takes(Recorder),
        )
        .requires(Recorder),
        card(
            "pruitt_pamphlets",
            Conspiracist,
            Unannounced,
            "Those pamphlets! The church is a front! For WHAT? For a BIGGER church! Wake up!",
            pick("Bin them", (0, -5, 5)).takes(Pamphlets),
            pick("Read them with him", (-10, 5, -5)),
        )
        .requires(Pamphlets),
        // ---------- Brother Josiah: loud, sure, wrong ----------
        card(
            "fanatic_pamphlets",
            Fanatic,
            Unannounced,
            "Two hundred pamphlets! Hand them out at the precinct! The Last Hour is NIGH! Also, bingo on Thursday.",
            pick("No pamphlets", (5, -5, 5)),
            pick("Take a stack", (-5, 5, -5)).gives(Pamphlets),
        )
        .excludes(Pamphlets),
        card(
            "fanatic_cat",
            Fanatic,
            Unannounced,
            "THE CAT! A familiar of the Serpent! Look into its eyes! ...Aww. No. Focus. EVIL!",
            pick("Hide the cat", (0, -5, 5)),
            pick("Let him pet it", (-5, 5, -10)),
        )
        .requires(Cat),
        card(
            "fanatic_rye",
            Fanatic,
            Unannounced,
            "I smell the Devil's rye on you! ...Is it the good stuff? Hand it over. For confiscation. Purposes.",
            pick("Not a drop", (5, 0, 5)),
            pick("Hand it over", (-5, -5, -5)).takes(Flask),
        )
        .requires(Flask),
        card(
            "fanatic_car",
            Fanatic,
            Unannounced,
            "I painted REPENT on your car. I meant to paint it on the hotel. They look alike in the dark.",
            pick("Arrest him", (5, 0, 10)),
            pick("Repent, then", (-10, 5, -10)),
        ),
        // ---------- Holloway's plaque: an honor nobody asked for ----------
        card(
            "officer_plaque",
            Officer,
            Summoned,
            "I made you Employee of the Month. Made the plaque myself. For the toenail work. Don't thank me. Okay, thank me.",
            pick("This isn't a thing", (0, -5, 0)),
            pick("Hang it up", (-5, 0, -5)).gives(Plaque),
        )
        .excludes(Plaque),
        card(
            "owner_plaque",
            HotelOwner,
            Unannounced,
            "May I borrow your plaque for the lobby? 'Employee of the Month' is the nicest thing said near 1313 in a century.",
            pick("It's mine", (5, 0, 0)),
            pick("Lend it to him", (-5, 0, -5)).takes(Plaque),
        )
        .requires(Plaque),
        card(
            "reporter_plaque",
            Reporter,
            Unannounced,
            "'PRECINCT HONORS TOENAIL.' Page six. Smile, detective. Or don't. Honestly, it's funnier if you don't.",
            pick("No comment", (-10, 0, 0)),
            pick("Pose with it", (5, 0, 5)),
        )
        .requires(Plaque),
        card(
            "fanatic_plaque",
            Fanatic,
            Unannounced,
            "A golden IDOL on your desk! 'Of the Month'? WHICH month? Do you even know what month it is, detective?",
            pick("Throw him out", (5, -5, 5)),
            pick("...October?", (0, 10, -5)),
        )
        .requires(Plaque),
        // ---------- Ostrowski's voucher: hospitality, weaponized ----------
        card(
            "owner_voucher",
            HotelOwner,
            Summoned,
            "A complimentary night in 1313. Nobody's ever complained. Nobody's ever checked out, either. Now leave my lobby.",
            pick("Tear it up", (5, -5, 0)),
            pick("Pocket it", (-5, 10, 0)).gives(Voucher),
        ),
        card(
            "officer_voucher",
            Officer,
            Unannounced,
            "Does the free night include breakfast? I'll come with you. I'm not scared. I'll bring a sandwich. Two sandwiches.",
            pick("Absolutely not", (0, 0, 5)),
            pick("Bring the sandwiches", (-5, 5, -10)),
        )
        .requires(Voucher),
        card(
            "reporter_voucher",
            Reporter,
            Unannounced,
            "A free night in 1313? Let me livestream it! You sleep, I narrate. If you die, it's a two-parter. Sweeps week!",
            pick("Find your own room", (-5, 0, 0)),
            pick("Hand it over", (15, 0, 0)).takes(Voucher),
        )
        .requires(Voucher),
        card(
            "pruitt_voucher",
            Conspiracist,
            Unannounced,
            "That voucher's a tracking device. Paper can be a device! Trade me. I'll give you a coupon for my newsletter.",
            pick("Keep it", (0, -5, 5)),
            pick("Trade", (-10, 0, -5)).takes(Voucher),
        )
        .requires(Voucher),
        // ---------- Quill's invoice: friendship, itemized ----------
        card(
            "occultist_invoice",
            Occultist,
            Summoned,
            "For services rendered: one consultation, two blessings, a mild exorcism of your car. Friend price: $1,313.",
            pick("I'm not paying", (5, -5, 5)),
            pick("Put it on my tab", (-5, 5, -5)).gives(Invoice),
        ),
        card(
            "officer_invoice",
            Officer,
            Summoned,
            "Accounting sent your invoice back. 'Exorcism' isn't a budget line. They suggest 'vehicle maintenance'.",
            pick("Fight accounting", (5, 0, 10)),
            pick("Vehicle maintenance", (-5, 0, -5)),
        )
        .requires(Invoice),
        card(
            "fanatic_invoice",
            Fanatic,
            Unannounced,
            "SIMONY! Selling blessings for coin! Give me that bill, I'll burn it! ...$1,313? For a car? Is he taking new clients?",
            pick("Keep it", (0, 0, 5)),
            pick("Let him burn it", (-5, -5, -5)).takes(Invoice),
        )
        .requires(Invoice),
        card(
            "occultist_late_fee",
            Occultist,
            Unannounced,
            "About my invoice. I added interest. And a late fee. And a fee for this reminder. We're friends. You understand.",
            pick("Friends don't invoice", (5, -5, 10)),
            pick("Pay him", (-10, 0, -10)).takes(Invoice),
        )
        .requires(Invoice)
        .after(Invoice, 6),
        // ---------- The Agent: rare, and it never gets easier ----------
        card(
            "agent_partner",
            Agent,
            Unannounced,
            "My partner went into 1313 in '87. Best man I ever knew. I still pour two cups of coffee in the morning.",
            pick("Say nothing", (0, -5, -10)),
            pick("Ask his name", (0, 10, -5)),
        )
        .requires(EReader)
        .after(EReader, 18),
        card(
            "agent_daughter",
            Agent,
            Unannounced,
            "My daughter asked what I do. I said I make sure bad things stay forgotten. She asked who remembers them.",
            pick("No one", (0, -10, -10)),
            pick("I do", (0, 10, -5)),
        )
        .requires(EReader)
        .after(EReader, 35),
        card(
            "agent_letter",
            Agent,
            Unannounced,
            "If you ever find a man named Eli Navarro in there, tell him I kept looking. Tell him I'm sorry I stopped.",
            pick("I can't promise", (5, -5, -10)),
            pick("I'll tell him", (0, 10, -5)),
        )
        .requires(EReader)
        .after(EReader, 55),
    ]
}
