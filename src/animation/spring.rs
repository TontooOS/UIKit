//! Spring physics.
//!
//! A damped harmonic oscillator in the spirit of Apple's
//! `UISpringTimingParameters`. Springs are the foundation of the "full
//! animation" feel in TontooOS: the dock bounce, the genie effect, and
//! window snaps all use spring curves instead of fixed easing functions.

use crate::animation::math::Vec2;

// ═══════════════════════════════════════════════════════════════
// Spring
// ═══════════════════════════════════════════════════════════════

/// A damped harmonic oscillator defined by mass, stiffness and damping.
///
/// The oscillator integrates a value toward a target using a semi-implicit
/// (symplectic) Euler step. With under-damped parameters the value overshoots
/// and bounces back, which produces the classic spring animation feel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring {
    /// Spring stiffness in units of force per unit distance (higher = snappier).
    pub stiffness: f32,
    /// Damping coefficient in units of force per unit velocity.
    pub damping: f32,
    /// Mass of the animated object. Heavier objects move more slowly.
    pub mass: f32,
    /// Relative velocity below which the spring is considered at rest.
    pub settle_threshold: f32,
}

impl Spring {
    /// A default, snappy spring tuned for UI animation.
    pub const fn new() -> Self {
        Self {
            stiffness: 180.0,
            damping: 25.0,
            mass: 1.0,
            settle_threshold: 0.01,
        }
    }

    /// A softer, slower spring.
    pub const fn soft() -> Self {
        Self {
            stiffness: 80.0,
            damping: 14.0,
            mass: 1.0,
            settle_threshold: 0.01,
        }
    }

    /// An over/under-damped bouncy spring.
    pub const fn bouncy() -> Self {
        Self {
            stiffness: 400.0,
            damping: 18.0,
            mass: 1.0,
            settle_threshold: 0.02,
        }
    }

    /// Configure stiffness (force per unit distance).
    pub const fn stiffness(mut self, stiffness: f32) -> Self {
        self.stiffness = stiffness;
        self
    }

    /// Configure damping (force per unit velocity).
    pub const fn damping(mut self, damping: f32) -> Self {
        self.damping = damping;
        self
    }

    /// Configure the oscillating mass.
    pub const fn mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    /// Advance the spring one integration step.
    ///
    /// Returns the new value for `current` and writes the new velocity into
    /// `velocity`.
    pub fn advance(&self, current: f32, target: f32, velocity: &mut f32, dt: f32) -> f32 {
        let dt = dt.clamp(0.0, 1.0 / 20.0);
        let acceleration = (self.stiffness * (target - current)) / self.mass - (self.damping * *velocity) / self.mass;
        *velocity += acceleration * dt;
        current + *velocity * dt
    }

    /// Advance a 2D position toward a target.
    pub fn advance_vec(&self, current: Vec2, target: Vec2, velocity: &mut Vec2, dt: f32) -> Vec2 {
        let x = self.advance(current.x, target.x, &mut velocity.x, dt);
        let y = self.advance(current.y, target.y, &mut velocity.y, dt);
        Vec2::new(x, y)
    }

    /// True when the spring is essentially at rest at the target.
    pub fn at_rest(&self, current: f32, target: f32, velocity: f32) -> bool {
        (target - current).abs() <= self.settle_threshold && velocity.abs() <= self.settle_threshold
    }

    /// True when a 2D spring is essentially at rest.
    pub fn at_rest_vec(&self, current: Vec2, target: Vec2, velocity: Vec2) -> bool {
        self.at_rest(current.x, target.x, velocity.x)
            && self.at_rest(current.y, target.y, velocity.y)
    }
}

impl Default for Spring {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════
// SpringPreset
// ═══════════════════════════════════════════════════════════════

/// Named spring presets matching common TontooOS animations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpringPreset {
    /// Default system spring (UI movement).
    Default,
    /// Fast, responsive snap (dock magnification, window snaps).
    Snappy,
    /// Gentle, elastic bounce (dock hover, notifications).
    Bouncy,
    /// Slow, smooth drift (badges, fade panels).
    Soft,
}

impl SpringPreset {
    /// Resolve the preset into a [`Spring`] configuration.
    pub fn spring(self) -> Spring {
        match self {
            SpringPreset::Default => Spring::new(),
            SpringPreset::Snappy => Spring::new().stiffness(300.0).damping(30.0),
            SpringPreset::Bouncy => Spring::bouncy(),
            SpringPreset::Soft => Spring::soft(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_converges_to_target() {
        let spring = Spring::new();
        let mut velocity = 0.0;
        let mut current = 0.0;
        let mut dt = 0.0;
        for _ in 0..1000 {
            current = spring.advance(current, 100.0, &mut velocity, 1.0 / 120.0);
            dt += 1.0 / 120.0;
            if spring.at_rest(current, 100.0, velocity) {
                current = 100.0;
                break;
            }
        }
        assert!(dt < 5.0, "spring should settle well under 5 seconds");
        assert!((current - 100.0).abs() < 1.0);
    }

    #[test]
    fn bouncy_spring_overshoots() {
        let spring = Spring::bouncy();
        let mut velocity = 0.0;
        let mut current = 0.0;
        let mut max = 0.0f32;
        for _ in 0..300 {
            current = spring.advance(current, 100.0, &mut velocity, 1.0 / 120.0);
            max = max.max(current);
        }
        // Under-damped spring must overshoot the target on its first swing.
        assert!(max > 100.0, "bouncy spring should overshoot, got max={max}");
    }

    #[test]
    fn spring_zero_dt_is_stable() {
        let spring = Spring::new();
        let mut velocity = 5.0;
        let current = spring.advance(10.0, 20.0, &mut velocity, 0.0);
        assert_eq!(current, 10.0);
        assert_eq!(velocity, 5.0);
    }

    #[test]
    fn presets_resolve() {
        assert_eq!(SpringPreset::Default.spring().stiffness, 180.0);
        assert!(SpringPreset::Snappy.spring().stiffness > SpringPreset::Default.spring().stiffness);
    }
}