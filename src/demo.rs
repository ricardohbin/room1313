//! Placeholder deck until the event format is designed.

use crate::engine::{Choice, Scene};

pub const START: &str = "intro";

fn scene(id: &str, speaker: &str, text: &str, left: (&str, &str), right: (&str, &str)) -> Scene {
    Scene {
        id: id.into(),
        speaker: speaker.into(),
        text: text.into(),
        left: Choice { label: left.0.into(), next: left.1.into() },
        right: Choice { label: right.0.into(), next: right.1.into() },
    }
}

pub fn scenes() -> Vec<Scene> {
    vec![
        scene(
            "intro",
            "Royal Advisor",
            "Your Majesty, the people are hungry. Shall we open the royal granaries?",
            ("Let them wait", "riot"),
            ("Open them", "feast"),
        ),
        scene(
            "riot",
            "Captain of the Guard",
            "A mob gathers at the gates. Do we hold the line?",
            ("Negotiate", "feast"),
            ("Hold the line", "war"),
        ),
        scene(
            "feast",
            "Jester",
            "The people cheer your name! Shall we throw a feast?",
            ("Save the coin", "war"),
            ("Feast!", "intro"),
        ),
        scene(
            "war",
            "General",
            "The neighbouring kingdom smells weakness. March on them first?",
            ("Send envoys", "intro"),
            ("March!", "riot"),
        ),
    ]
}
