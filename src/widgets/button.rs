//! Button widget — clickable button using GtkButton with CSS styling.
//!
//! This is the UIKit equivalent of `UIButton` in Apple's UIKit.
//!
//! ```rust,no_run
//! use uikit::prelude::*;
//!
//! let button = Button::new("Click Me")
//!     .background(Color::from_hex("#FF6B2B").unwrap())
//!     .text_color(Color::WHITE)
//!     .on_click(|| println!("Clicked!"));
//!
//! let view = View::new(button)
//!     .with_frame(16.0, 16.0, 120.0, 44.0);
//! ```

use crate::style::{Color, Padding, Rect};
use crate::view::{View, ViewContent};
use crate::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self, Button as GtkButton};
use std::sync::Arc;

pub struct Button {
    id: WidgetId,
    label: String,
    background: Color,
    text_color: Color,
    corner_radius: f32,
    h_padding: f32,
    v_padding: f32,
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id: next_widget_id(),
            label: label.into(),
            background: Color::new(0.047, 0.522, 0.937, 1.0),
            text_color: Color::WHITE,
            corner_radius: 8.0,
            h_padding: 16.0,
            v_padding: 8.0,
            on_click: None,
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

    pub fn glass(milkiness: f32, alpha: f32) -> Self {
        Self {
            background: Color::new(milkiness, milkiness, milkiness, alpha),
            ..Self::new("")
        }
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn padding(mut self, horizontal: f32, vertical: f32) -> Self {
        self.h_padding = horizontal;
        self.v_padding = vertical;
        self
    }

    pub fn on_click(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(handler));
        self
    }

    pub fn on_custom(mut self, action: impl Into<String>) -> Self {
        let action_name = action.into();
        self.on_click = Some(Arc::new(move || {
            crate::app::dispatch_custom(&action_name);
        }));
        self
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    /// Create a View wrapping this Button.
    pub fn to_view(self, x: f32, y: f32, width: f32, height: f32) -> View {
        View::new(self).with_frame(x, y, width, height)
    }
}

impl ViewContent for Button {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let btn = GtkButton::with_label(&self.label);

        let bg_hex = format!(
            "#{:02x}{:02x}{:02x}",
            (self.background.r * 255.0) as u8,
            (self.background.g * 255.0) as u8,
            (self.background.b * 255.0) as u8,
        );
        let fg_hex = format!(
            "#{:02x}{:02x}{:02x}",
            (self.text_color.r * 255.0) as u8,
            (self.text_color.g * 255.0) as u8,
            (self.text_color.b * 255.0) as u8,
        );

        let css = format!(
            "button {{
                background: {};
                color: {};
                border-radius: {}px;
                padding: {}px {}px;
                font-family: 'SF Pro Display';
                font-size: 13px;
                min-width: {}px;
                min-height: {}px;
            }}
            button:hover {{
                filter: brightness(1.1);
            }}",
            bg_hex,
            fg_hex,
            self.corner_radius,
            self.v_padding,
            self.h_padding,
            frame.width.max(0.0) as i32,
            frame.height.max(0.0) as i32,
        );
        crate::widget::apply_css(&btn, &css);

        if let Some(handler) = &self.on_click {
            let handler = handler.clone();
            btn.connect_clicked(move |_| {
                handler();
            });
        }

        btn.upcast()
    }

    fn can_become_first_responder(&self) -> bool {
        true
    }
}

impl Widget for Button {
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
        let btn = GtkButton::with_label(&self.label);

        let bg_hex = format!(
            "#{:02x}{:02x}{:02x}",
            (self.background.r * 255.0) as u8,
            (self.background.g * 255.0) as u8,
            (self.background.b * 255.0) as u8,
        );
        let fg_hex = format!(
            "#{:02x}{:02x}{:02x}",
            (self.text_color.r * 255.0) as u8,
            (self.text_color.g * 255.0) as u8,
            (self.text_color.b * 255.0) as u8,
        );

        let css = format!(
            "button {{
                background: {};
                color: {};
                border-radius: {}px;
                padding: {}px {}px;
                font-family: 'SF Pro Display';
                font-size: 13px;
                min-width: {}px;
                min-height: {}px;
            }}
            button:hover {{
                filter: brightness(1.1);
            }}",
            bg_hex,
            fg_hex,
            self.corner_radius,
            self.v_padding,
            self.h_padding,
            self.position.width.unwrap_or(0.0).max(0.0) as i32,
            self.position.height.unwrap_or(0.0).max(0.0) as i32,
        );
        crate::widget::apply_css(&btn, &css);

        if let Some(handler) = &self.on_click {
            let handler = handler.clone();
            btn.connect_clicked(move |_| {
                handler();
            });
        }

        if self.position_mode == PositionMode::Absolute {
            let mut css = String::from("button {");
            if let Some(x) = self.position.x {
                css.push_str(&format!("margin-left: {}px;", x));
            }
            if let Some(y) = self.position.y {
                css.push_str(&format!("margin-top: {}px;", y));
            }
            css.push('}');
            crate::widget::apply_css(&btn, &css);
        }

        btn.upcast()
    }

    fn is_interactive(&self) -> bool {
        true
    }

    fn padding(&self) -> Padding {
        Padding::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_builder() {
        let btn = Button::new("OK")
            .background(Color::RED)
            .text_color(Color::WHITE)
            .corner_radius(12.0);
        assert_eq!(btn.label(), "OK");
        assert_eq!(btn.corner_radius, 12.0);
    }

    #[test]
    fn button_absolute() {
        let btn = Button::new("Go").at(10.0, 20.0).size(100.0, 40.0);
        assert_eq!(btn.position_mode(), PositionMode::Absolute);
        assert_eq!(btn.position.x, Some(10.0));
        assert_eq!(btn.position.width, Some(100.0));
    }
}
