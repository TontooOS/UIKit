//! Style types for TontooUIKit.
//!
//! Core types: `Color`, `Font`, `Padding`, `Size`, `Point`, `Alignment`.

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════
// Color
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const TRANSPARENT: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: 1.0 }
    }

    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: a as f32 / 255.0 }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::from_rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::from_rgba(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Convert to CSS hex string like "#ff0000".
    pub fn to_hex(&self) -> String {
        format!(
            "#{:02x}{:02x}{:02x}",
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
        )
    }

    /// Convert to CSS rgba() string.
    pub fn to_css(&self) -> String {
        format!(
            "rgba({:.0}, {:.0}, {:.0}, {:.2})",
            self.r * 255.0,
            self.g * 255.0,
            self.b * 255.0,
            self.a,
        )
    }
}

impl Color {
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);
    pub const YELLOW: Self = Self::new(1.0, 1.0, 0.0, 1.0);
    pub const CYAN: Self = Self::new(0.0, 1.0, 1.0, 1.0);
    pub const MAGENTA: Self = Self::new(1.0, 0.0, 1.0, 1.0);
    pub const GRAY: Self = Self::new(0.5, 0.5, 0.5, 1.0);
    pub const LIGHT_GRAY: Self = Self::new(0.75, 0.75, 0.75, 1.0);
    pub const DARK_GRAY: Self = Self::new(0.25, 0.25, 0.25, 1.0);
    pub const ACCENT: Self = Self::new(0.047, 0.522, 0.937, 1.0);
}

// ═══════════════════════════════════════════════════════════════
// Font
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FontWeight {
    Thin, Light, Regular, Medium, Semibold, Bold, Heavy,
}

impl Default for FontWeight {
    fn default() -> Self { Self::Regular }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Font {
    pub family: String,
    pub size: f32,
    pub weight: FontWeight,
}

impl Font {
    pub fn new(family: impl Into<String>, size: f32) -> Self {
        Self { family: family.into(), size, weight: FontWeight::Regular }
    }

    pub fn system(size: f32) -> Self {
        Self::new("SF Pro Display", size)
    }

    pub fn monospace(size: f32) -> Self {
        Self::new("SF Mono", size)
    }
}

impl Default for Font {
    fn default() -> Self { Self::system(13.0) }
}

// ═══════════════════════════════════════════════════════════════
// Padding
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Padding {
    pub const ZERO: Self = Self { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0 };

    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }

    pub const fn all(value: f32) -> Self {
        Self { top: value, right: value, bottom: value, left: value }
    }

    pub const fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self { top: vertical, right: horizontal, bottom: vertical, left: horizontal }
    }

    pub const fn horizontal(value: f32) -> Self {
        Self { top: 0.0, right: value, bottom: 0.0, left: value }
    }

    pub const fn vertical(value: f32) -> Self {
        Self { top: value, right: 0.0, bottom: value, left: 0.0 }
    }

    pub fn horizontal_total(&self) -> f32 { self.left + self.right }
    pub fn vertical_total(&self) -> f32 { self.top + self.bottom }
}

impl Default for Padding {
    fn default() -> Self { Self::ZERO }
}

// ═══════════════════════════════════════════════════════════════
// Size & Point
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Self = Self { width: 0.0, height: 0.0 };
    pub const fn new(width: f32, height: f32) -> Self { Self { width, height } }
}

impl Default for Size {
    fn default() -> Self { Self::ZERO }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const fn new(x: f32, y: f32) -> Self { Self { x, y } }
    pub fn translate(self, dx: f32, dy: f32) -> Self { Self { x: self.x + dx, y: self.y + dy } }
}

impl Default for Point {
    fn default() -> Self { Self::ZERO }
}

// ═══════════════════════════════════════════════════════════════
// Rect
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };

    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn origin(&self) -> Point {
        Point::new(self.x, self.y)
    }

    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

impl Default for Rect {
    fn default() -> Self { Self::ZERO }
}

// ═══════════════════════════════════════════════════════════════
// Alignment
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HAlignment { Leading, Center, Trailing }

impl Default for HAlignment {
    fn default() -> Self { Self::Leading }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VAlignment { Top, Center, Bottom }

impl Default for VAlignment {
    fn default() -> Self { Self::Top }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Alignment {
    pub horizontal: HAlignment,
    pub vertical: VAlignment,
}

impl Alignment {
    pub const CENTER: Self = Self { horizontal: HAlignment::Center, vertical: VAlignment::Center };
    pub const LEADING_TOP: Self = Self { horizontal: HAlignment::Leading, vertical: VAlignment::Top };
}

// ═══════════════════════════════════════════════════════════════
// Constraints (kept for API compat, but not used by GTK4 backend)
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Constraints {
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
}

impl Constraints {
    pub const UNBOUNDED: Self = Self { min_width: None, max_width: None, min_height: None, max_height: None };

    pub const fn tight(max_width: f32, max_height: f32) -> Self {
        Self { min_width: None, max_width: Some(max_width), min_height: None, max_height: Some(max_height) }
    }

    pub const fn bounded(width: f32, height: f32) -> Self {
        Self { min_width: Some(width), max_width: Some(width), min_height: Some(height), max_height: Some(height) }
    }

    pub fn max_w(&self) -> f32 { self.max_width.unwrap_or(f32::INFINITY) }
    pub fn max_h(&self) -> f32 { self.max_height.unwrap_or(f32::INFINITY) }
    pub fn min_w(&self) -> f32 { self.min_width.unwrap_or(0.0) }
    pub fn min_h(&self) -> f32 { self.min_height.unwrap_or(0.0) }
    pub fn clamp_width(&self, w: f32) -> f32 { w.clamp(self.min_w(), self.max_w()) }
    pub fn clamp_height(&self, h: f32) -> f32 { h.clamp(self.min_h(), self.max_h()) }
}

impl Default for Constraints {
    fn default() -> Self { Self::UNBOUNDED }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_hex() {
        assert_eq!(Color::from_hex("#FF5500"), Some(Color::from_rgb(255, 85, 0)));
        assert_eq!(Color::from_hex("invalid"), None);
    }

    #[test]
    fn padding_symmetric() {
        let p = Padding::symmetric(10.0, 20.0);
        assert_eq!(p.top, 10.0);
        assert_eq!(p.left, 20.0);
        assert_eq!(p.horizontal_total(), 40.0);
    }
}
