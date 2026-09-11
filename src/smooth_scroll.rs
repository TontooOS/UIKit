//! Smooth scrolling for `gtk::ScrolledWindow`.
//!
//! GTK scrolls discrete mouse-wheel ticks instantly by one step increment per
//! tick, which feels stepped: every wheel notch jumps the content with no
//! animation. This module intercepts scroll events with a capture-phase
//! [`gtk::EventControllerScroll`] and animates the adjustments toward a
//! target value with an ease-out-cubic frame animation, giving macOS/iOS-like
//! smooth scrolling.
//!
//! Behavior:
//!
//! - Discrete wheel steps expand to [`WHEEL_STEP_PX`] pixels per tick (bounded
//!   by [`MIN_STEP_PX`] / [`MAX_STEP_PX`], derived from the adjustment step
//!   increment when sensible) and animate over [`SCROLL_DURATION`].
//! - Touchpad (smooth/pixel) deltas are applied 1:1 so the native smooth
//!   scrolling feel is preserved; the animation target follows along so mixed
//!   wheel + touchpad input never fights.
//! - Rapid successive ticks retarget the running animation instead of stacking
//!   timers, so fast spins travel further but stay fluid.
//! - Kinetic touch scrolling and overlay scrollbars are enabled on every
//!   window this helper is applied to.
//!
//! Apply with [`apply_smooth_scrolling`] on every `ScrolledWindow` the toolkit
//! creates (`ScrollView`, `ListView`, `App` scroll wrapper).

use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Pixels travelled per discrete mouse-wheel tick.
pub const WHEEL_STEP_PX: f64 = 64.0;
/// Lower bound for a wheel step derived from the adjustment step increment.
pub const MIN_STEP_PX: f64 = 24.0;
/// Upper bound for a wheel step derived from the adjustment step increment.
pub const MAX_STEP_PX: f64 = 120.0;
/// Duration of the settle animation per scroll burst.
pub const SCROLL_DURATION: Duration = Duration::from_millis(200);

/// Animation state shared between the scroll handler and the tick callback.
#[derive(Debug, Default)]
struct ScrollAnim {
    v_from: f64,
    v_to: f64,
    h_from: f64,
    h_to: f64,
    start: Option<Instant>,
    running: bool,
}

/// Clamp a scroll target into the scrollable range of an adjustment.
///
/// `upper - page_size` is the highest reachable value; when the content fits
/// the viewport the range collapses to `lower`.
pub fn clamp_target(value: f64, lower: f64, upper: f64, page_size: f64) -> f64 {
    let max = (upper - page_size).max(lower);
    value.clamp(lower, max)
}

/// Ease-out cubic: fast start, gentle settle. Pure for tests.
pub fn ease_out_cubic(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t) * (1.0 - t)
}

/// Interpolated adjustment value for `elapsed` since the animation start.
/// Returns `to` once [`SCROLL_DURATION`] has passed. Pure for tests.
pub fn animated_value(from: f64, to: f64, elapsed: Duration) -> f64 {
    if elapsed >= SCROLL_DURATION {
        return to;
    }
    let t = elapsed.as_secs_f64() / SCROLL_DURATION.as_secs_f64();
    from + (to - from) * ease_out_cubic(t)
}

/// Wheel travel in pixels for one discrete tick, derived from the adjustment
/// step increment when it is in a sensible range. Pure for tests.
pub fn wheel_step(step_increment: f64) -> f64 {
    if step_increment > 0.0 {
        step_increment.clamp(MIN_STEP_PX, MAX_STEP_PX)
    } else {
        WHEEL_STEP_PX
    }
}

/// Enable smooth animated scrolling on a `ScrolledWindow`.
///
/// Installs a capture-phase scroll controller that animates discrete wheel
/// ticks and passes touchpad pixel deltas through 1:1. Safe to call on any
/// `ScrolledWindow`; also enables kinetic and overlay scrolling.
pub fn apply_smooth_scrolling(scrolled: &gtk::ScrolledWindow) {
    scrolled.set_kinetic_scrolling(true);
    scrolled.set_overlay_scrolling(true);

    let state = Rc::new(RefCell::new(ScrollAnim::default()));
    let controller = gtk::EventControllerScroll::new(
        gtk::EventControllerScrollFlags::BOTH_AXES
            | gtk::EventControllerScrollFlags::DISCRETE,
    );
    // Capture runs before ScrolledWindow's internal bubble-phase controller,
    // so returning Stop below replaces the instant jump with our animation.
    controller.set_propagation_phase(gtk::PropagationPhase::Capture);

    let weak = scrolled.downgrade();
    controller.connect_scroll(move |controller, dx, dy| {
        let Some(scrolled) = weak.upgrade() else {
            return glib::Propagation::Proceed;
        };
        if dx == 0.0 && dy == 0.0 {
            return glib::Propagation::Proceed;
        }
        let vadj = scrolled.vadjustment();
        let hadj = scrolled.hadjustment();

        if matches!(controller.unit(), gtk::gdk::ScrollUnit::Surface) {
            // Touchpad: apply pixel deltas directly (native smooth feel) and
            // keep the animation target in sync so a running wheel animation
            // does not yank the content back.
            let v = clamp_target(vadj.value() + dy, vadj.lower(), vadj.upper(), vadj.page_size());
            let h = clamp_target(hadj.value() + dx, hadj.lower(), hadj.upper(), hadj.page_size());
            vadj.set_value(v);
            hadj.set_value(h);
            let mut s = state.borrow_mut();
            s.v_from = v;
            s.v_to = v;
            s.h_from = h;
            s.h_to = h;
            return glib::Propagation::Stop;
        }

        // Mouse wheel: expand discrete steps to pixels and animate.
        let dy_px = dy * wheel_step(vadj.step_increment());
        let dx_px = dx * wheel_step(hadj.step_increment());
        {
            let mut s = state.borrow_mut();
            if !s.running {
                s.v_to = vadj.value();
                s.h_to = hadj.value();
            }
            // Retarget from the live position so mid-flight ticks stay
            // responsive instead of easing from a stale origin.
            s.v_from = vadj.value();
            s.h_from = hadj.value();
            s.v_to = clamp_target(s.v_to + dy_px, vadj.lower(), vadj.upper(), vadj.page_size());
            s.h_to = clamp_target(s.h_to + dx_px, hadj.lower(), hadj.upper(), hadj.page_size());
            s.start = Some(Instant::now());
            if s.running {
                return glib::Propagation::Stop;
            }
            s.running = true;
        }

        let tick_state = state.clone();
        let tick_weak = scrolled.downgrade();
        scrolled.add_tick_callback(move |scrolled, _| {
            let Some(_) = tick_weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            let mut s = tick_state.borrow_mut();
            let elapsed = s.start.map(|t| t.elapsed()).unwrap_or(SCROLL_DURATION);
            let finished = elapsed >= SCROLL_DURATION;
            let vadj = scrolled.vadjustment();
            let hadj = scrolled.hadjustment();
            // Re-clamp every frame so content resizes mid-animation cannot
            // push the adjustment out of range.
            let v = clamp_target(
                animated_value(s.v_from, s.v_to, elapsed),
                vadj.lower(),
                vadj.upper(),
                vadj.page_size(),
            );
            let h = clamp_target(
                animated_value(s.h_from, s.h_to, elapsed),
                hadj.lower(),
                hadj.upper(),
                hadj.page_size(),
            );
            vadj.set_value(v);
            hadj.set_value(h);
            if finished {
                // A retarget resets `start`, so reaching the full duration
                // always means the latest target was applied exactly.
                s.running = false;
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });

        glib::Propagation::Stop
    });
    scrolled.add_controller(controller);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_target_collapses_when_content_fits() {
        assert_eq!(clamp_target(500.0, 0.0, 400.0, 600.0), 0.0);
        assert_eq!(clamp_target(-10.0, 0.0, 1000.0, 400.0), 0.0);
    }

    #[test]
    fn clamp_target_limits_to_scroll_range() {
        assert_eq!(clamp_target(9999.0, 0.0, 1000.0, 400.0), 600.0);
        assert_eq!(clamp_target(300.0, 0.0, 1000.0, 400.0), 300.0);
    }

    #[test]
    fn ease_out_cubic_monotone_settle() {
        assert_eq!(ease_out_cubic(0.0), 0.0);
        assert_eq!(ease_out_cubic(1.0), 1.0);
        let mid = ease_out_cubic(0.5);
        assert!(mid > 0.5 && mid < 1.0);
        assert!(ease_out_cubic(0.25) < mid);
    }

    #[test]
    fn animated_value_snaps_after_duration() {
        assert_eq!(animated_value(0.0, 100.0, Duration::from_millis(0)), 0.0);
        assert_eq!(animated_value(0.0, 100.0, SCROLL_DURATION), 100.0);
        assert_eq!(
            animated_value(0.0, 100.0, SCROLL_DURATION + Duration::from_millis(50)),
            100.0
        );
        let mid = animated_value(0.0, 100.0, Duration::from_millis(100));
        assert!(mid > 50.0 && mid < 100.0);
    }

    #[test]
    fn wheel_step_bounds() {
        assert_eq!(wheel_step(0.0), WHEEL_STEP_PX);
        assert_eq!(wheel_step(-5.0), WHEEL_STEP_PX);
        assert_eq!(wheel_step(5.0), MIN_STEP_PX);
        assert_eq!(wheel_step(500.0), MAX_STEP_PX);
        assert_eq!(wheel_step(48.0), 48.0);
    }
}
