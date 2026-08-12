//! The physics animator.
//!
//! [`Animator`] owns a set of [`DynamicItem`]s (positions, velocities, scale,
//! opacity) plus a list of composable [`Behavior`]s. Advancing a frame applies
//! every behavior to every item, then integrates velocity into position.
//!
//! Items are pure data — they do not know about GTK. To drive real widgets,
//! bind an item to a `gtk::Widget` with
//! [`AnimatedWidget::bind`](crate::animation::widget::AnimatedWidget::bind).

use crate::animation::behaviors::Behavior;
use crate::animation::math::{Rect, Vec2};
use crate::animation::spring::Spring;
use crate::style::Size;

// ═══════════════════════════════════════════════════════════════
// DynamicItem
// ═══════════════════════════════════════════════════════════════

/// A physics body inside an [`Animator`].
///
/// Tracks position/velocity (2D) plus spring-animated scale and opacity, so a
/// single item covers movement *and* the grow/fade effects used by the dock
/// and notifications.
#[derive(Debug, Clone)]
pub struct DynamicItem {
    /// Position of the item's center.
    pub position: Vec2,
    /// Velocity in "pixels per second".
    pub velocity: Vec2,
    /// Approximate size used for collision.
    pub size: Size,
    /// Mass used in collision resolution.
    pub mass: f32,
    /// Elasticity / restitution for collisions (`0..1`).
    pub elasticity: f32,
    /// Whether the item is fixed in place (no integration).
    pub resting: bool,
    /// Current scale factor (`1.0` = natural size).
    pub scale: f32,
    /// Target scale the spring animates toward.
    pub scale_target: f32,
    /// Current scale velocity (pixels-per-second of the scale value).
    pub scale_velocity: f32,
    /// Spring used for the scale animation.
    pub scale_spring: Option<Spring>,
    /// Current opacity (`0..1`).
    pub opacity: f32,
    /// Target opacity the spring animates toward.
    pub opacity_target: f32,
    /// Current opacity velocity.
    pub opacity_velocity: f32,
    /// Spring used for the opacity animation.
    pub opacity_spring: Option<Spring>,
}

impl DynamicItem {
    /// Create a resting item at `position` with the given size.
    pub fn new(position: Vec2, size: Size) -> Self {
        Self {
            position,
            velocity: Vec2::new(0.0, 0.0),
            size,
            mass: 1.0,
            elasticity: 0.6,
            resting: true,
            scale: 1.0,
            scale_target: 1.0,
            scale_velocity: 0.0,
            scale_spring: None,
            opacity: 1.0,
            opacity_target: 1.0,
            opacity_velocity: 0.0,
            opacity_spring: None,
        }
    }

    /// Spring-animate scale to `target`.
    pub fn animate_scale(&mut self, target: f32) {
        let spring = self.scale_spring.unwrap_or_default();
        self.scale_spring = Some(spring);
        self.scale_target = target;
        self.scale_velocity = 0.0;
    }

    /// Spring-animate opacity to `target`.
    pub fn animate_opacity(&mut self, target: f32) {
        let spring = self.opacity_spring.unwrap_or_default();
        self.opacity_spring = Some(spring);
        self.opacity_target = target;
        self.opacity_velocity = 0.0;
    }

    /// Set the scale (instant, no animation).
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
        self.scale_target = scale;
        self.scale_velocity = 0.0;
    }

    /// Set the opacity (instant, no animation).
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
        self.opacity_target = opacity;
        self.opacity_velocity = 0.0;
    }

    /// True when the position is not moving.
    pub fn is_settled(&self) -> bool {
        self.velocity.length() < 0.01
    }

    /// Wake the item (used to re-enable integration).
    pub fn wake(&mut self) {
        self.resting = false;
    }
}

// ═══════════════════════════════════════════════════════════════
// Animator
// ═══════════════════════════════════════════════════════════════

/// Owns items and behaviors and advances them each frame.
#[derive(Default)]
pub struct Animator {
    items: Vec<DynamicItem>,
    behaviors: Vec<Box<dyn Behavior>>,
    /// Global bounds for boundary collisions (applied every tick).
    pub bounds: Option<Rect>,
}

impl Animator {
    /// Create an empty animator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set global boundary bounds and add a boundary collision behavior.
    pub fn set_bounds(&mut self, bounds: Option<Rect>) {
        self.bounds = bounds;
        self.add_behavior(
            crate::animation::behaviors::CollisionBehavior::new(bounds),
        );
    }

    /// Add an item and return its index. Indices are stable and used by
    /// target-specific behaviors.
    pub fn add_item(&mut self, item: DynamicItem) -> usize {
        self.items.push(item);
        self.items.len() - 1
    }

    /// Number of items.
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Borrow an item by index.
    pub fn item(&self, index: usize) -> Option<&DynamicItem> {
        self.items.get(index)
    }

    /// Mutably borrow an item by index.
    pub fn item_mut(&mut self, index: usize) -> Option<&mut DynamicItem> {
        self.items.get_mut(index)
    }

    /// Add a composable behavior.
    pub fn add_behavior(&mut self, behavior: impl Behavior + 'static) {
        self.behaviors.push(Box::new(behavior));
    }

    /// Advance the simulation by `dt` seconds.
    ///
    /// Applies each active behavior, integrates velocities, then advances
    /// scale/opacity springs. Finished one-shot behaviors are removed.
    pub fn tick(&mut self, dt: f32) {
        // Phase 1: apply behaviors.
        for behavior in self.behaviors.iter_mut() {
            behavior.apply(&mut self.items, dt);
        }
        self.behaviors.retain(|b| b.active());

        // Phase 2: integrate positions.
        for item in self.items.iter_mut() {
            if item.resting {
                item.velocity = Vec2::new(0.0, 0.0);
                continue;
            }
            item.position.x += item.velocity.x * dt;
            item.position.y += item.velocity.y * dt;
        }

        // Phase 3: advance scale/opacity springs.
        for item in self.items.iter_mut() {
            if let Some(spring) = item.scale_spring {
                item.scale = spring.advance(
                    item.scale,
                    item.scale_target,
                    &mut item.scale_velocity,
                    dt,
                );
                if spring.at_rest(item.scale, item.scale_target, item.scale_velocity) {
                    item.scale = item.scale_target;
                    item.scale_velocity = 0.0;
                    item.scale_spring = None;
                }
            }
            if let Some(spring) = item.opacity_spring {
                item.opacity = spring.advance(
                    item.opacity,
                    item.opacity_target,
                    &mut item.opacity_velocity,
                    dt,
                );
                if spring.at_rest(item.opacity, item.opacity_target, item.opacity_velocity) {
                    item.opacity = item.opacity_target;
                    item.opacity_velocity = 0.0;
                    item.opacity_spring = None;
                }
            }
        }
    }

    /// True when any item is moving or any spring is unsettled.
    pub fn is_running(&self) -> bool {
        self.items.iter().any(|item| {
            let moving = !item.resting && item.velocity.length() > 0.01;
            let scale_busy = item
                .scale_spring
                .is_some_and(|_| (item.scale - item.scale_target).abs() > 0.001);
            let opacity_busy = item
                .opacity_spring
                .is_some_and(|_| (item.opacity - item.opacity_target).abs() > 0.001);
            moving || scale_busy || opacity_busy
        })
    }
}