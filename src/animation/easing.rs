//! Easing functions.
//!
//! Standard easing curves used by tween-style animations. Spring physics live
//! in [`spring`](crate::animation::spring); these curves are used when a fixed
//! duration is preferred (fade-in, slide panels, notification banners).

// ═══════════════════════════════════════════════════════════════
// Easing
// ═══════════════════════════════════════════════════════════════

/// Named easing curves. `t` is expected in `[0, 1]`; curves are normalized so
/// they also return values in `[0, 1]` (some overshoot slightly, see [`Back`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    /// No easing; linear interpolation.
    Linear,
    /// Quadratic ease-in.
    QuadIn,
    /// Quadratic ease-out.
    QuadOut,
    /// Quadratic ease-in-out.
    QuadInOut,
    /// Cubic ease-in.
    CubicIn,
    /// Cubic ease-out.
    CubicOut,
    /// Cubic ease-in-out.
    CubicInOut,
    /// Sine ease-in.
    SineIn,
    /// Sine ease-out.
    SineOut,
    /// Sine ease-in-out.
    SineInOut,
    /// Ease-out with a slight overshoot (elastic feel).
    BackOut,
    /// Ease-in starting with a backward pull.
    BackIn,
    /// Overshoots past the target then settles (bounce).
    BounceOut,
}

impl Easing {
    /// Apply the easing curve to a progress value in `[0, 1]`.
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::QuadIn => t * t,
            Easing::QuadOut => t * (2.0 - t),
            Easing::QuadInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            Easing::CubicIn => t * t * t,
            Easing::CubicOut => {
                let u = t - 1.0;
                u * u * u + 1.0
            }
            Easing::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let u = 2.0 * t - 2.0;
                    0.5 * u * u * u + 1.0
                }
            }
            Easing::SineIn => 1.0 - (t * std::f32::consts::PI / 2.0).cos(),
            Easing::SineOut => (t * std::f32::consts::PI / 2.0).sin(),
            Easing::SineInOut => 0.5 - 0.5 * (std::f32::consts::PI * t).cos(),
            Easing::BackOut => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                let u = t - 1.0;
                1.0 + c3 * u * u * u + c1 * u * u
            }
            Easing::BackIn => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * t * t * t - c1 * t * t
            }
            Easing::BounceOut => {
                if t < 1.0 / 2.75 {
                    7.5625 * t * t
                } else if t < 2.0 / 2.75 {
                    let t = t - 1.5 / 2.75;
                    7.5625 * t * t + 0.75
                } else if t < 2.5 / 2.75 {
                    let t = t - 2.25 / 2.75;
                    7.5625 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / 2.75;
                    7.5625 * t * t + 0.984375
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Tween
// ═══════════════════════════════════════════════════════════════

/// A fixed-duration, single-value tween driven by an [`Easing`] curve.
///
/// Use this for deterministic animations where a spring is undesirable
/// (e.g. notification banners that must exit after exactly 2.5 seconds).
#[derive(Debug, Clone)]
pub struct Tween {
    from: f32,
    to: f32,
    duration: f32,
    easing: Easing,
    elapsed: f32,
}

impl Tween {
    /// Create a tween from `from` to `to` over `duration` seconds.
    pub fn new(from: f32, to: f32, duration: f32) -> Self {
        Self {
            from,
            to,
            duration: duration.max(0.0001),
            easing: Easing::QuadOut,
            elapsed: 0.0,
        }
    }

    /// Override the easing curve.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Progress in `[0, 1]` (0 = start, 1 = done).
    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    /// Current value with the easing curve applied.
    pub fn value(&self) -> f32 {
        let p = self.progress();
        let eased = self.easing.apply(p);
        self.from + (self.to - self.from) * eased
    }

    /// Advance the tween by `dt` seconds. Returns whether it is finished.
    pub fn advance(&mut self, dt: f32) -> bool {
        self.elapsed = (self.elapsed + dt).min(self.duration);
        self.elapsed >= self.duration
    }

    /// True when the tween has reached its duration.
    pub fn finished(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Target value.
    pub fn target(&self) -> f32 {
        self.to
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easing_endpoints() {
        for easing in [
            Easing::Linear,
            Easing::QuadIn,
            Easing::CubicOut,
            Easing::SineOut,
            Easing::BounceOut,
        ] {
            assert!((easing.apply(0.0) - 0.0).abs() < 1e-4, "{easing:?}");
            assert!((easing.apply(1.0) - 1.0).abs() < 1e-4, "{easing:?}");
        }
    }

    #[test]
    fn back_out_overshoots() {
        assert!(Easing::BackOut.apply(0.5) > 1.0);
    }

    #[test]
    fn tween_interpolates() {
        let mut tween = Tween::new(0.0, 10.0, 1.0);
        assert_eq!(tween.value(), 0.0);
        assert!(tween.advance(0.5));
        assert!(!tween.finished());
        assert!(tween.value() > 0.0 && tween.value() < 10.0);
        assert!(tween.advance(0.6));
        assert!(tween.finished());
        assert_eq!(tween.value(), 10.0);
    }
}