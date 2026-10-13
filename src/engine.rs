use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct Choice {
    pub label: String,
    pub next: String,
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub id: String,
    pub speaker: String,
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

/// Holds the scene graph and the player's position in it.
/// The graph is validated up front, so navigation can never fail.
pub struct Engine {
    scenes: HashMap<String, Scene>,
    current: String,
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
            history: Vec::new(),
        })
    }

    pub fn current(&self) -> &Scene {
        &self.scenes[&self.current]
    }

    /// Applies a choice on the current scene and moves to the next one.
    pub fn choose(&mut self, side: Side) -> &Scene {
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
            speaker: "x".into(),
            text: "x".into(),
            left: Choice { label: "no".into(), next: left.into() },
            right: Choice { label: "yes".into(), next: right.into() },
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
