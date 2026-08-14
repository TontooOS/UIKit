//! # Animation
//!
//! GTK4 binding layer for the [`uikitdynamics`](https://docs.rs/uikitdynamics)
//! physics engine.
//!
//! The pure physics (springs, easing, behaviors, animator) lives in the
//! `uikitdynamics` crate; this module re-exports it and adds the GTK-bound
//! [`AnimatedWidget`] which maps physics items onto real widgets every frame.
//!
//! ```
//! use uikit::animation::prelude::*;
//! use uikitdynamics::Size;
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

pub mod widget;

pub use uikitdynamics::animator::{Animator, DynamicItem};
pub use uikitdynamics::behaviors::{
    AttachmentBehavior, Behavior, CollisionBehavior, GravityBehavior, ItemProperties,
    PushBehavior, SnapBehavior,
};
pub use uikitdynamics::easing::{Easing, Tween};
pub use uikitdynamics::math::{Rect, Size, Vec2, v};
pub use uikitdynamics::spring::{Spring, SpringPreset};
pub use widget::AnimatedWidget;

/// Re-exports for convenient glob imports.
pub mod prelude {
    pub use super::{
        AnimatedWidget, Animator, AttachmentBehavior, Behavior, CollisionBehavior,
        DynamicItem, Easing, GravityBehavior, ItemProperties, PushBehavior, Rect,
        SnapBehavior, Spring, SpringPreset, Tween, Vec2, v,
    };
    pub use uikitdynamics::Size;
}