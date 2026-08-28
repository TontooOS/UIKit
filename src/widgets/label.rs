//! Label widget — displays text using GtkLabel with Pango rendering.
//!
//! This is the UIKit equivalent of `UILabel` in Apple's UIKit.
//!
//! ```rust,no_run
//! use uikit::prelude::*;
//!
//! let label = Label::new("Hello, TontooOS!")
//!     .font_size(24.0)
//!     .bold()
//!     .color(Color::WHITE);
//!
//! let view = View::new(label)
//!     .with_frame(16.0, 16.0, 200.0, 30.0);
//! ```

use crate::style::{Color, Font, FontWeight, Padding, Rect, Size};
use crate::view::{View, ViewContent};
use crate::widget::{Widget, WidgetId, Position, PositionMode, next_widget_id};
use gtk::prelude::*;
use gtk::{self, Label as GtkLabel};

pub struct Label {
    id: WidgetId,
    content: String,
    font: Font,
    color: Color,
    max_width: Option<f32>,
    position_mode: PositionMode,
    position: Position,
}

impl Label {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            id: next_widget_id(),
            content: content.into(),
            font: Font::default(),
            color: Color::new(0.92, 0.92, 0.94, 1.0),
            max_width: None,
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
        self.position.width = Some(width);
        self.position.height = Some(height);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.position.width = Some(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.position.height = Some(height);
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font.size = size;
        self
    }

    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font.family = family.into();
        self
    }

    pub fn bold(mut self) -> Self {
        self.font.weight = FontWeight::Bold;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    fn weight_to_pango(&self) -> &str {
        match self.font.weight {
            FontWeight::Thin => "ultralight",
            FontWeight::Light => "light",
            FontWeight::Regular => "normal",
            FontWeight::Medium => "medium",
            FontWeight::Semibold => "semibold",
            FontWeight::Bold => "bold",
            FontWeight::Heavy => "heavy",
        }
    }

    fn color_hex(&self) -> String {
        format!(
            "#{:02x}{:02x}{:02x}",
            (self.color.r * 255.0) as u8,
            (self.color.g * 255.0) as u8,
            (self.color.b * 255.0) as u8,
        )
    }

    /// Create a View wrapping this Label.
    pub fn to_view(self, x: f32, y: f32, width: f32, height: f32) -> View {
        View::new(self).with_frame(x, y, width, height)
    }
}

impl ViewContent for Label {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let label = GtkLabel::new(Some(&self.content));

        let font_desc = format!("{} {} {}", self.font.family, self.font.size, self.weight_to_pango());
        label.set_markup(&format!(
            "<span font_desc=\"{}\" foreground=\"{}\">{}</span>",
            font_desc,
            self.color_hex(),
            glib::markup_escape_text(&self.content),
        ));

        if let Some(max_w) = self.max_width {
            label.set_width_chars((max_w / (self.font.size * 0.6)) as i32);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        }

        label.set_halign(gtk::Align::Start);
        label.set_valign(gtk::Align::Center);

        if frame.width > 0.0 {
            label.set_width_request(frame.width as i32);
        }
        if frame.height > 0.0 {
            label.set_height_request(frame.height as i32);
        }

        label.upcast()
    }

    fn size_that_fits(&self, available: Size) -> Size {
        // Estimate label size based on content length and font size
        let char_width = self.font.size * 0.6;
        let estimated_width = (self.content.len() as f32) * char_width;
        Size::new(
            estimated_width.min(available.width),
            self.font.size * 1.2,
        )
    }
}

impl Widget for Label {
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
        let label = GtkLabel::new(Some(&self.content));

        let font_desc = format!("{} {} {}", self.font.family, self.font.size, self.weight_to_pango());
        label.set_markup(&format!(
            "<span font_desc=\"{}\" foreground=\"{}\">{}</span>",
            font_desc,
            self.color_hex(),
            glib::markup_escape_text(&self.content),
        ));

        if let Some(max_w) = self.max_width {
            label.set_width_chars((max_w / (self.font.size * 0.6)) as i32);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        }

        label.set_halign(gtk::Align::Start);
        label.set_valign(gtk::Align::Center);

        if self.position_mode == PositionMode::Absolute {
            let mut css = String::from("label {");
            if let Some(x) = self.position.x {
                css.push_str(&format!("margin-left: {}px;", x));
            }
            if let Some(y) = self.position.y {
                css.push_str(&format!("margin-top: {}px;", y));
            }
            if let Some(w) = self.position.width {
                css.push_str(&format!("min-width: {}px;", w));
            }
            if let Some(h) = self.position.height {
                css.push_str(&format!("min-height: {}px;", h));
            }
            css.push('}');
            crate::widget::apply_css(&label, &css);
        }

        label.upcast()
    }

    fn is_interactive(&self) -> bool {
        false
    }

    fn padding(&self) -> Padding {
        Padding::ZERO
    }
}

// Type alias for backward compatibility
pub type Text = Label;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_builder() {
        let label = Label::new("Hello")
            .font_size(24.0)
            .bold()
            .color(Color::WHITE);
        assert_eq!(label.content(), "Hello");
        assert_eq!(label.font.size, 24.0);
        assert_eq!(label.font.weight, FontWeight::Bold);
    }

    #[test]
    fn label_absolute_position() {
        let label = Label::new("Hi").at(50.0, 100.0).size(200.0, 30.0);
        assert_eq!(label.position_mode(), PositionMode::Absolute);
        assert_eq!(label.position.x, Some(50.0));
        assert_eq!(label.position.y, Some(100.0));
        assert_eq!(label.position.width, Some(200.0));
        assert_eq!(label.position.height, Some(30.0));
    }

    #[test]
    fn label_to_view() {
        let view = Label::new("Test").to_view(10.0, 20.0, 100.0, 30.0);
        assert_eq!(view.frame().x, 10.0);
        assert_eq!(view.frame().y, 20.0);
        assert_eq!(view.width(), 100.0);
        assert_eq!(view.height(), 30.0);
    }
}
