//! Math helpers for the animation engine.
//!
//! Provides a 2D vector alias over [`Point`](crate::style::Point) with the
//! arithmetic operators and normalization helpers the physics engine needs,
//! plus an axis-aligned `Rect` used for collision bounds.

use crate::style::Point;
use std::ops::{Add, Mul, Sub};

// ═══════════════════════════════════════════════════════════════
// Vec2
// ═══════════════════════════════════════════════════════════════

/// A 2D vector. This is an alias of [`Point`](crate::style::Point) so it
/// stays serializable and interchangeable with widget positions.
pub type Vec2 = Point;

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, scalar: f32) -> Vec2 {
        Vec2::new(self.x * scalar, self.y * scalar)
    }
}

impl Vec2 {
    /// Euclidean length of the vector.
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Unit-length vector in the same direction. Returns [`Vec2::ZERO`]
    /// when the length is (near) zero.
    pub fn normalized(&self) -> Vec2 {
        let len = self.length();
        if len <= f32::EPSILON {
            Vec2::ZERO
        } else {
            Vec2::new(self.x / len, self.y / len)
        }
    }

    /// Dot product with another vector.
    pub fn dot(&self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Scale both components by `scalar`.
    pub fn scale(&self, scalar: f32) -> Vec2 {
        Vec2::new(self.x * scalar, self.y * scalar)
    }
}

/// Convenience constructor: `v(10.0, 20.0)`.
pub fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

// ═══════════════════════════════════════════════════════════════
// Rect
// ═══════════════════════════════════════════════════════════════

/// An axis-aligned rectangle in 2D space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Minimum corner (top-left).
    pub fn min(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Maximum corner (bottom-right).
    pub fn max(&self) -> Vec2 {
        Vec2::new(self.x + self.width, self.y + self.height)
    }

    /// Whether `p` lies on or inside the rectangle.
    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= self.x
            && p.y >= self.y
            && p.x <= self.x + self.width
            && p.y <= self.y + self.height
    }

    /// Clamp `p` to the rectangle (inclusive).
    pub fn clamp(&self, p: Vec2) -> Vec2 {
        Vec2::new(
            p.x.clamp(self.x, self.x + self.width),
            p.y.clamp(self.y, self.y + self.height),
        )
    }
}