//! Icon widget — displays an icon (PNG or SVG) with optional tinting.
//!
//! This is a convenience wrapper around ImageView for icon assets.
//!
//! ```rust,no_run
//! use uikit::prelude::*;
//!
//! let icon = Icon::new("/usr/share/icons/setting.png")
//!     .size(24.0, 24.0)
//!     .tint(Color::WHITE);
//!
//! let view = View::new(icon)
//!     .with_frame(16.0, 16.0, 24.0, 24.0);
//! ```

use crate::style::{Color, Padding, Rect, Size};
use crate::view::{View, ViewContent};
use crate::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self, Picture};

pub struct Icon {
    id: WidgetId,
    path: String,
    width: f32,
    height: f32,
    tint: Option<Color>,
    opacity: f32,
    position_mode: PositionMode,
    position: Position,
}

impl Icon {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            id: next_widget_id(),
            path: path.into(),
            width: 24.0,
            height: 24.0,
            tint: None,
            opacity: 1.0,
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

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn tint(mut self, color: Color) -> Self {
        self.tint = Some(color);
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    /// Create a View wrapping this Icon.
    pub fn to_view(self, x: f32, y: f32) -> View {
        let w = self.width;
        let h = self.height;
        View::new(self).with_frame(x, y, w, h)
    }
}

impl ViewContent for Icon {
    fn render(&self, _frame: Rect) -> gtk::Widget {
        let file = gtk::gio::File::for_path(&self.path);
        let picture = Picture::for_file(&file);

        picture.set_width_request(self.width as i32);
        picture.set_height_request(self.height as i32);
        picture.set_opacity(self.opacity as f64);
        picture.set_content_fit(gtk::ContentFit::Contain);

        // Apply tint via CSS if specified
        if let Some(_tint) = self.tint {
            let css = format!(
                "picture {{ filter: invert(1) sepia(1) saturate(5) hue-rotate(0deg) brightness(1.2); }}"
            );
            crate::widget::apply_css(&picture, &css);
        }

        picture.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        Size::new(self.width, self.height)
    }
}

impl Widget for Icon {
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
        let file = gtk::gio::File::for_path(&self.path);
        let picture = Picture::for_file(&file);

        picture.set_width_request(self.width as i32);
        picture.set_height_request(self.height as i32);
        picture.set_opacity(self.opacity as f64);
        picture.set_content_fit(gtk::ContentFit::Contain);

        picture.upcast()
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
    fn icon_builder() {
        let icon = Icon::new("/icons/setting.png")
            .size(32.0, 32.0)
            .opacity(0.9);
        assert_eq!(icon.path(), "/icons/setting.png");
        assert_eq!(icon.width, 32.0);
        assert_eq!(icon.height, 32.0);
        assert_eq!(icon.opacity, 0.9);
    }
}
