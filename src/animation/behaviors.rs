//! Physics behaviors.
//!
//! Modeled after Apple's UIKit Dynamics. Behaviors are composable force
//! modifiers attached to an [`Animator`](crate::animation::animator::Animator):
//!
//! - [`GravityBehavior`] applies a constant acceleration.
//! - [`CollisionBehavior`] resolves boundary and item-item collisions.
//! - [`AttachmentBehavior`] springs an item toward a point (or `length` away).
//! - [`PushBehavior`] applies a one-shot or continuous impulse.
//! - [`SnapBehavior`] springs an item exactly to a target point.
//! - [`ItemProperties`] tunes mass, elasticity and resistance per item.

use crate::animation::animator::DynamicItem;
use crate::animation::math::{Rect, Vec2};
use crate::animation::spring::Spring;

// ═══════════════════════════════════════════════════════════════
// Behavior trait
// ═══════════════════════════════════════════════════════════════

/// A composable physics behavior applied to a set of items.
pub trait Behavior {
    /// Apply the behavior to `items`. `dt` is the frame delta in seconds.
    fn apply(&mut self, items: &mut [DynamicItem], dt: f32);

    /// Whether the behavior still has work to do. Behaviors that complete
    /// (e.g. one-shot pushes) return `false` and are removed by the animator.
    fn active(&self) -> bool {
        true
    }
}

// ═══════════════════════════════════════════════════════════════
// GravityBehavior
// ═══════════════════════════════════════════════════════════════

/// Applies a constant acceleration to every item (e.g. `v(0, 980)`).
pub struct GravityBehavior {
    /// Acceleration vector, typically expressed in "pixels per second squared".
    pub vector: Vec2,
    /// Optional per-frame multiplier used to re-run gravity numerically.
    pub strength: f32,
}

impl GravityBehavior {
    pub fn new(vector: Vec2) -> Self {
        Self { vector, strength: 1.0 }
    }

    /// Scale the gravitational force.
    pub fn strength(mut self, strength: f32) -> Self {
        self.strength = strength;
        self
    }
}

impl Default for GravityBehavior {
    /// Default gravity pulling straight down.
    fn default() -> Self {
        Self::new(Vec2::new(0.0, 980.0))
    }
}

impl Behavior for GravityBehavior {
    fn apply(&mut self, items: &mut [DynamicItem], dt: f32) {
        if self.vector.length() == 0.0 {
            return;
        }
        for item in items.iter_mut() {
            if item.resting {
                continue;
            }
            item.velocity.x += self.vector.x * self.strength * dt;
            item.velocity.y += self.vector.y * self.strength * dt;
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// CollisionBehavior
// ═══════════════════════════════════════════════════════════════

/// Resolves collisions against an optional boundary rectangle and between
/// items. Items bounce according to their per-item elasticity.
pub struct CollisionBehavior {
    /// Bounding rectangle items are confined to. `None` disables boundaries.
    pub bounds: Option<Rect>,
    /// Whether items collide with each other.
    pub collide_items: bool,
}

impl CollisionBehavior {
    pub fn new(bounds: Option<Rect>) -> Self {
        Self { bounds, collide_items: true }
    }

    /// Confine items to a rectangle.
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Enable or disable item-to-item collision.
    pub fn collide_items(mut self, enabled: bool) -> Self {
        self.collide_items = enabled;
        self
    }
}

impl Default for CollisionBehavior {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Behavior for CollisionBehavior {
    fn apply(&mut self, items: &mut [DynamicItem], _dt: f32) {
        // Boundary bounce.
        if let Some(bounds) = self.bounds {
            for item in items.iter_mut() {
                let half_w = item.size.width / 2.0;
                let half_h = item.size.height / 2.0;
                let min = bounds.min();
                let max = bounds.max();
                if item.position.x - half_w < min.x {
                    item.position.x = min.x + half_w;
                    item.velocity.x = item.velocity.x.abs() * item.elasticity;
                } else if item.position.x + half_w > max.x {
                    item.position.x = max.x - half_w;
                    item.velocity.x = -item.velocity.x.abs() * item.elasticity;
                }
                if item.position.y - half_h < min.y {
                    item.position.y = min.y + half_h;
                    item.velocity.y = item.velocity.y.abs() * item.elasticity;
                } else if item.position.y + half_h > max.y {
                    item.position.y = max.y - half_h;
                    item.velocity.y = -item.velocity.y.abs() * item.elasticity;
                }
            }
        }

        // Item-item collisions: simple circle/circle with per-item elasticity.
        if self.collide_items {
            for i in 0..items.len() {
                for j in (i + 1)..items.len() {
                    let (a, b);
                    let (left, right) = items.split_at_mut(j);
                    a = &mut left[i];
                    b = &mut right[0];
                    Self::resolve_item_collision(a, b);
                }
            }
        }
    }
}

impl CollisionBehavior {
    fn resolve_item_collision(a: &mut DynamicItem, b: &mut DynamicItem) {
        if a.resting && b.resting {
            return;
        }
        let delta = b.position - a.position;
        let dist_sq = delta.x * delta.x + delta.y * delta.y;
        let min_dist = (a.size.width + b.size.width) / 2.0;
        if dist_sq == 0.0 || dist_sq >= min_dist * min_dist {
            return;
        }

        let dist = dist_sq.sqrt();
        let normal = Vec2::new(delta.x / dist, delta.y / dist);
        let overlap = min_dist - dist;

        let elasticity = a.elasticity.max(b.elasticity).clamp(0.0, 1.0);
        let mass_sum = a.mass + b.mass;
        let a_ratio = if mass_sum == 0.0 { 0.5 } else { b.mass / mass_sum };
        let b_ratio = 1.0 - a_ratio;

        a.position.x -= normal.x * overlap * a_ratio;
        a.position.y -= normal.y * overlap * a_ratio;
        b.position.x += normal.x * overlap * b_ratio;
        b.position.y += normal.y * overlap * b_ratio;

        let rel = a.velocity - b.velocity;
        let vn = rel.dot(normal);
        if vn > 0.0 {
            return;
        }
        a.velocity.x -= normal.x * vn * elasticity;
        a.velocity.y -= normal.y * vn * elasticity;
        b.velocity.x += normal.x * vn * elasticity;
        b.velocity.y += normal.y * vn * elasticity;
    }
}

// ═══════════════════════════════════════════════════════════════
// AttachmentBehavior
// ═══════════════════════════════════════════════════════════════

/// Springs an item toward an anchor point. Use `length` for a fixed-distance
/// tether ("rubber band"), or `length: 0` to pull toward the point itself.
pub struct AttachmentBehavior {
    /// Index of the item this behavior affects.
    pub item_index: Option<usize>,
    /// Anchor point the item pulls toward.
    pub anchor: Vec2,
    /// Rest distance from the anchor. `0` means spring to the anchor.
    pub length: f32,
    /// Spring configuration.
    pub spring: Spring,
}

impl AttachmentBehavior {
    /// Attach `item_index` to `anchor`.
    pub fn new(item_index: usize, anchor: Vec2) -> Self {
        Self {
            item_index: Some(item_index),
            anchor,
            length: 0.0,
            spring: Spring::new().damping(14.0),
        }
    }

    /// Sets the rest length (rubber-band distance).
    pub fn length(mut self, length: f32) -> Self {
        self.length = length;
        self
    }

    /// Override the spring tuning.
    pub fn spring(mut self, spring: Spring) -> Self {
        self.spring = spring;
        self
    }
}

impl Behavior for AttachmentBehavior {
    fn apply(&mut self, items: &mut [DynamicItem], dt: f32) {
        for (index, item) in items.iter_mut().enumerate() {
            if let Some(target) = self.item_index {
                if index != target {
                    continue;
                }
            }
            if item.resting {
                continue;
            }
            let delta = self.anchor - item.position;
            let dist = delta.length();
            if dist == 0.0 {
                continue;
            }
            let direction = delta.scale(1.0 / dist);
            // Desired point: `length` away from the anchor along the direction.
            let desired = self.anchor + direction.scale(-self.length);
            item.position = self.spring.advance_vec(
                item.position,
                desired,
                &mut item.velocity,
                dt,
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// PushBehavior
// ═══════════════════════════════════════════════════════════════

/// Applies a one-shot impulse (velocity delta) or a continuous acceleration.
pub struct PushBehavior {
    /// Index of the item this behavior affects.
    pub item_index: Option<usize>,
    /// Impulse magnitude (`velocity += direction * magnitude`).
    pub magnitude: f32,
    /// Whether the push re-applies every frame (`true`) or once (`false`).
    pub continuous: bool,
    /// Push direction (normalized internally).
    pub direction: Vec2,
    consumed: bool,
}

impl PushBehavior {
    /// Push `item_index` along `direction` with the given impulse.
    pub fn new(item_index: usize, direction: Vec2, magnitude: f32) -> Self {
        Self {
            item_index: Some(item_index),
            magnitude,
            continuous: false,
            direction: direction.normalized(),
            consumed: false,
        }
    }

    /// Re-apply the push every frame instead of once.
    pub fn continuous(mut self, continuous: bool) -> Self {
        self.continuous = continuous;
        self
    }
}

impl Behavior for PushBehavior {
    fn apply(&mut self, items: &mut [DynamicItem], dt: f32) {
        if self.consumed && !self.continuous {
            return;
        }
        for (index, item) in items.iter_mut().enumerate() {
            if let Some(target) = self.item_index {
                if index != target {
                    continue;
                }
            }
            if item.resting {
                continue;
            }
            let dv = if self.continuous {
                self.direction * (self.magnitude * dt)
            } else {
                self.direction * self.magnitude
            };
            item.velocity.x += dv.x;
            item.velocity.y += dv.y;
        }
        self.consumed = true;
    }

    fn active(&self) -> bool {
        self.continuous || !self.consumed
    }
}

// ═══════════════════════════════════════════════════════════════
// SnapBehavior
// ═══════════════════════════════════════════════════════════════

/// Springs an item exactly onto a target point, settling with a configurable
/// damping ratio (mirrors UIKit's `UISnapBehavior`).
pub struct SnapBehavior {
    /// Index of the item this behavior affects.
    pub item_index: Option<usize>,
    /// Target position.
    pub target: Vec2,
    /// Spring tuning used for the snap.
    pub spring: Spring,
}

impl SnapBehavior {
    /// Snap `item_index` to `target`.
    pub fn new(item_index: usize, target: Vec2) -> Self {
        Self {
            item_index: Some(item_index),
            target,
            spring: Spring::new().stiffness(190.0).damping(32.0),
        }
    }

    /// Override the spring tuning.
    pub fn spring(mut self, spring: Spring) -> Self {
        self.spring = spring;
        self
    }
}

impl Behavior for SnapBehavior {
    fn apply(&mut self, items: &mut [DynamicItem], dt: f32) {
        for (index, item) in items.iter_mut().enumerate() {
            if let Some(target) = self.item_index {
                if index != target {
                    continue;
                }
            }
            item.position = self.spring.advance_vec(
                item.position,
                self.target,
                &mut item.velocity,
                dt,
            );
            if self
                .spring
                .at_rest_vec(item.position, self.target, item.velocity)
            {
                item.position = self.target;
                item.velocity = Vec2::new(0.0, 0.0);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// ItemProperties
// ═══════════════════════════════════════════════════════════════

/// A non-physics "behavior" that just exposes per-item tuning. Kept for API
/// symmetry with UIKit Dynamics; the animator exposes the same knobs directly.
pub struct ItemProperties {
    /// Mass used in collision resolution (frac mass).
    pub mass: f32,
    /// Elasticity / restitution of collisions.
    pub elasticity: f32,
    /// Linear resistance applied each frame (`velocity *= (1 - resistance * dt)`).
    pub resistance: f32,
}

impl Default for ItemProperties {
    fn default() -> Self {
        Self { mass: 1.0, elasticity: 0.6, resistance: 0.0 }
    }
}

impl Behavior for ItemProperties {
    fn apply(&mut self, items: &mut [DynamicItem], dt: f32) {
        for item in items.iter_mut() {
            if self.resistance > 0.0 {
                let factor = 1.0 - (self.resistance * dt).min(1.0);
                item.velocity.x *= factor;
                item.velocity.y *= factor;
            }
            item.mass = self.mass;
            item.elasticity = self.elasticity.clamp(0.0, 1.0);
        }
    }
}