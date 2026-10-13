use crate::engine::Side;

/// Tunables for how a drag maps to card movement and decisions.
#[derive(Debug, Clone, Copy)]
pub struct SwipeConfig {
    /// Horizontal distance (px) past which releasing commits a choice.
    pub threshold: f64,
    /// Tilt (deg) reached at `rotation_span` px of horizontal drag.
    pub max_rotation_deg: f64,
    pub rotation_span: f64,
    /// Vertical movement is damped so the card mostly slides sideways.
    pub vertical_damping: f64,
}

impl Default for SwipeConfig {
    fn default() -> Self {
        Self {
            threshold: 100.0,
            max_rotation_deg: 15.0,
            rotation_span: 250.0,
            vertical_damping: 0.25,
        }
    }
}

impl SwipeConfig {
    pub fn rotation(&self, dx: f64) -> f64 {
        (dx / self.rotation_span).clamp(-1.0, 1.0) * self.max_rotation_deg
    }

    /// Which side the card is leaning towards and how close (0..=1) it is to committing.
    pub fn leaning(&self, dx: f64) -> Option<(Side, f64)> {
        if dx == 0.0 {
            return None;
        }
        let side = if dx < 0.0 { Side::Left } else { Side::Right };
        Some((side, (dx.abs() / self.threshold).min(1.0)))
    }

    /// The choice made if the card is released at `dx`.
    pub fn decide(&self, dx: f64) -> Option<Side> {
        match self.leaning(dx) {
            Some((side, s)) if s >= 1.0 => Some(side),
            _ => None,
        }
    }
}

/// An in-progress drag, in screen coordinates.
#[derive(Debug, Clone, Copy)]
pub struct Drag {
    origin: (f64, f64),
    pos: (f64, f64),
}

impl Drag {
    pub fn start(x: f64, y: f64) -> Self {
        Self { origin: (x, y), pos: (x, y) }
    }

    pub fn update(&mut self, x: f64, y: f64) {
        self.pos = (x, y);
    }

    pub fn dx(&self) -> f64 {
        self.pos.0 - self.origin.0
    }

    pub fn dy(&self) -> f64 {
        self.pos.1 - self.origin.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decides_only_past_threshold() {
        let cfg = SwipeConfig::default();
        assert_eq!(cfg.decide(0.0), None);
        assert_eq!(cfg.decide(99.0), None);
        assert_eq!(cfg.decide(-99.0), None);
        assert_eq!(cfg.decide(100.0), Some(Side::Right));
        assert_eq!(cfg.decide(-150.0), Some(Side::Left));
    }

    #[test]
    fn leaning_strength_is_clamped() {
        let cfg = SwipeConfig::default();
        assert_eq!(cfg.leaning(-50.0), Some((Side::Left, 0.5)));
        assert_eq!(cfg.leaning(500.0), Some((Side::Right, 1.0)));
    }

    #[test]
    fn rotation_is_clamped() {
        let cfg = SwipeConfig::default();
        assert_eq!(cfg.rotation(10_000.0), cfg.max_rotation_deg);
        assert_eq!(cfg.rotation(-10_000.0), -cfg.max_rotation_deg);
    }

    #[test]
    fn drag_tracks_offset() {
        let mut d = Drag::start(10.0, 20.0);
        d.update(-30.0, 25.0);
        assert_eq!((d.dx(), d.dy()), (-40.0, 5.0));
    }
}
