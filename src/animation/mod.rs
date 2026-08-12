//! # Animation
//!
//! A UIKit-Dynamics-style physics and animation engine.
//!
//! Provides spring-physics timing, easing curves, composable behaviors
//! (gravity, collisions, attachments, pushes, snaps) and a frame-driven
//! [`Animator`] that can bind directly to GTK4 widgets.
//!
//! ```
//! use tontoo_uikit::animation::prelude::*;
//! use tontoo_uikit::style::Size;
//!
//! let mut animator = Animator::new();
//! let item = animator.add_item(DynamicItem::new(v(0.0, 0.0), Size::new(60.0, 60.0)));
//! let mut item = animator.item_mut(item).unwrap();
//! item.wake();
//! item.velocity = v(0.0, 0.0);
//! drop(item);
//!
//! animator.add_behavior(GravityBehavior::new(v(0.0, 980.0)));
//! animator.set_bounds(Some(Rect::new(0.0, 0.0, 800.0, 600.0)));
//!
//! for _ in 0..120 {
//!     animator.tick(1.0 / 60.0);
//! }
//! assert!(animator.is_running());
//! ```

pub mod animator;
pub mod behaviors;
pub mod easing;
pub mod math;
pub mod spring;
pub mod widget;

pub use animator::{Animator, DynamicItem};
pub use behaviors::{
    AttachmentBehavior, Behavior, CollisionBehavior, GravityBehavior, ItemProperties,
    PushBehavior, SnapBehavior,
};
pub use easing::{Easing, Tween};
pub use math::{Rect, Vec2, v};
pub use spring::{Spring, SpringPreset};
pub use widget::AnimatedWidget;

/// Re-exports for convenient glob imports.
pub mod prelude {
    pub use super::{
        AnimatedWidget, Animator, AttachmentBehavior, Behavior, CollisionBehavior,
        DynamicItem, Easing, GravityBehavior, ItemProperties, PushBehavior, Rect,
        SnapBehavior, Spring, SpringPreset, Tween, Vec2, v,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Size;

    #[test]
    fn gravity_pulls_item_down() {
        let mut animator = Animator::new();
        let mut item = DynamicItem::new(Vec2::new(0.0, 0.0), Size::new(20.0, 20.0));
        item.wake();
        animator.add_item(item);
        animator.add_behavior(GravityBehavior::default());

        let before = animator.item(0).unwrap().position.y;
        animator.tick(1.0 / 60.0);
        let after = animator.item(0).unwrap().position.y;
        assert!(after > before, "gravity should pull the item down");
    }

    #[test]
    fn bounds_keep_item_inside() {
        let mut animator = Animator::new();
        let mut item = DynamicItem::new(Vec2::new(50.0, 50.0), Size::new(20.0, 20.0));
        item.wake();
        item.velocity = Vec2::new(0.0, 10000.0);
        animator.add_item(item);
        animator.set_bounds(Some(Rect::new(0.0, 0.0, 100.0, 100.0)));
        for _ in 0..600 {
            animator.tick(1.0 / 60.0);
        }
        let p = animator.item(0).unwrap().position;
        assert!(p.y <= 100.0 + 1.0, "item escaped bounds: {p:?}");
    }

    #[test]
    fn snap_settles_on_target() {
        let mut animator = Animator::new();
        let mut item = DynamicItem::new(Vec2::new(0.0, 0.0), Size::new(30.0, 30.0));
        item.wake();
        let index = animator.add_item(item);
        animator.add_behavior(SnapBehavior::new(index, Vec2::new(200.0, 150.0)));
        for _ in 0..300 {
            animator.tick(1.0 / 60.0);
        }
        let p = animator.item(index).unwrap().position;
        assert!((p.x - 200.0).abs() < 1.0, "snap x off: {p:?}");
        assert!((p.y - 150.0).abs() < 1.0, "snap y off: {p:?}");
    }
}