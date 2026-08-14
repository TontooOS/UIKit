//! Separator widget — a horizontal or vertical divider line.
//!
//! This is the UIKit equivalent of `UIView` with a thin background,
//! or `UISeparator` in SwiftUI.
//!
//! ```rust,no_run
//! use uikit::prelude::*;
//!
//! let separator = Separator::horizontal()
//!     .color(Color::from_hex("#3a3a3d").unwrap());
//!
//! let view = View::new(separator)
//!     .with_frame(0.0, 0.0, 400.0, 1.0);
//! ```

use crate::style::{Color, Padding, Rect, Size};
use crate::view::{View, ViewContent};
use crate::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeparatorOrientation {
    Horizontal,
    Vertical,
}

pub struct Separator {
    id: WidgetId,
    orientation: SeparatorOrientation,
    color: Color,
    thickness: f32,
    position_mode: PositionMode,
    position: Position,
}

impl Separator {
    pub fn horizontal() -> Self {
        Self {
            id: next_widget_id(),
            orientation: SeparatorOrientation::Horizontal,
            color: Color::from_hex("#3a3a3d").unwrap_or(Color::GRAY),
            thickness: 1.0,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn vertical() -> Self {
        Self {
            id: next_widget_id(),
            orientation: SeparatorOrientation::Vertical,
            color: Color::from_hex("#3a3a3d").unwrap_or(Color::GRAY),
            thickness: 1.0,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.position_mode = PositionMode::Absolute;
        self.position.x = Some(x);
        self.position.y = Some(y);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    /// Create a View wrapping this Separator.
    pub fn to_view(self, x: f32, y: f32, width: f32, height: f32) -> View {
        View::new(self).with_frame(x, y, width, height)
    }
}

impl Default for Separator {
    fn default() -> Self {
        Self::horizontal()
    }
}

impl ViewContent for Separator {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let separator = gtk::Separator::new(match self.orientation {
            SeparatorOrientation::Horizontal => gtk::Orientation::Horizontal,
            SeparatorOrientation::Vertical => gtk::Orientation::Vertical,
        });

        let color_hex = format!(
            "#{:02x}{:02x}{:02x}",
            (self.color.r * 255.0) as u8,
            (self.color.g * 255.0) as u8,
            (self.color.b * 255.0) as u8,
        );

        let css = match self.orientation {
            SeparatorOrientation::Horizontal => format!(
                "separator {{
                    min-height: {}px;
                    background-color: {};
                }}",
                self.thickness, color_hex
            ),
            SeparatorOrientation::Vertical => format!(
                "separator {{
                    min-width: {}px;
                    background-color: {};
                }}",
                self.thickness, color_hex
            ),
        };
        crate::widget::apply_css(&separator, &css);

        if frame.width > 0.0 && self.orientation == SeparatorOrientation::Horizontal {
            separator.set_width_request(frame.width as i32);
        }
        if frame.height > 0.0 && self.orientation == SeparatorOrientation::Vertical {
            separator.set_height_request(frame.height as i32);
        }

        separator.upcast()
    }

    fn size_that_fits(&self, available: Size) -> Size {
        match self.orientation {
            SeparatorOrientation::Horizontal => Size::new(available.width, self.thickness),
            SeparatorOrientation::Vertical => Size::new(self.thickness, available.height),
        }
    }
}

impl Widget for Separator {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn position_mode(&self) -> PositionMode {
        self.position_mode
    }

    fn position(&self) -> Position {
        self.position
    }

    fn to_gtk(&self) -> gtk::Widget {
        let separator = gtk::Separator::new(match self.orientation {
            SeparatorOrientation::Horizontal => gtk::Orientation::Horizontal,
            SeparatorOrientation::Vertical => gtk::Orientation::Vertical,
        });

        let color_hex = format!(
            "#{:02x}{:02x}{:02x}",
            (self.color.r * 255.0) as u8,
            (self.color.g * 255.0) as u8,
            (self.color.b * 255.0) as u8,
        );

        let css = match self.orientation {
            SeparatorOrientation::Horizontal => format!(
                "separator {{
                    min-height: {}px;
                    background-color: {};
                }}",
                self.thickness, color_hex
            ),
            SeparatorOrientation::Vertical => format!(
                "separator {{
                    min-width: {}px;
                    background-color: {};
                }}",
                self.thickness, color_hex
            ),
        };
        crate::widget::apply_css(&separator, &css);

        separator.upcast()
    }

    fn is_interactive(&self) -> bool {
        false
    }

    fn padding(&self) -> Padding {
        Padding::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separator_horizontal() {
        let sep = Separator::horizontal();
        assert_eq!(sep.orientation, SeparatorOrientation::Horizontal);
        assert_eq!(sep.thickness, 1.0);
    }

    #[test]
    fn separator_vertical() {
        let sep = Separator::vertical();
        assert_eq!(sep.orientation, SeparatorOrientation::Vertical);
    }

    #[test]
    fn separator_custom_color() {
        let sep = Separator::horizontal().color(Color::RED).thickness(2.0);
        assert_eq!(sep.color, Color::RED);
        assert_eq!(sep.thickness, 2.0);
    }
}
