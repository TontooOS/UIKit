//! Toggle widget — an on/off switch using GtkSwitch.
//!
//! This is the UIKit equivalent of `UISwitch` in Apple's UIKit.
//!
//! ```rust,no_run
//! use uikit::prelude::*;
//!
//! let toggle = Toggle::new()
//!     .is_on(true)
//!     .on_change(|is_on| println!("Toggle: {}", is_on));
//!
//! let view = View::new(toggle)
//!     .with_frame(16.0, 16.0, 51.0, 31.0);
//! ```

use crate::style::{Color, Padding, Rect, Size};
use crate::view::{View, ViewContent};
use crate::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self, Switch};
use std::sync::Arc;

pub struct Toggle {
    id: WidgetId,
    is_on: bool,
    on_color: Color,
    off_color: Color,
    on_change: Option<Arc<dyn Fn(bool) + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl Toggle {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            is_on: false,
            on_color: Color::from_rgb(52, 199, 89),
            off_color: Color::from_rgb(120, 120, 128),
            on_change: None,
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

    pub fn is_on(mut self, is_on: bool) -> Self {
        self.is_on = is_on;
        self
    }

    pub fn on_color(mut self, color: Color) -> Self {
        self.on_color = color;
        self
    }

    pub fn off_color(mut self, color: Color) -> Self {
        self.off_color = color;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(bool) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }

    pub fn is_toggled(&self) -> bool {
        self.is_on
    }

    pub fn to_view(self, x: f32, y: f32) -> View {
        View::new(self).with_frame(x, y, 51.0, 31.0)
    }

    fn build_switch(&self) -> gtk::Widget {
        let switch = Switch::new();
        switch.set_active(self.is_on);
        switch.set_hexpand(false);
        switch.set_valign(gtk::Align::Center);

        let on_hex = format!(
            "#{:02x}{:02x}{:02x}",
            (self.on_color.r * 255.0) as u8,
            (self.on_color.g * 255.0) as u8,
            (self.on_color.b * 255.0) as u8,
        );
        let off_hex = format!(
            "#{:02x}{:02x}{:02x}",
            (self.off_color.r * 255.0) as u8,
            (self.off_color.g * 255.0) as u8,
            (self.off_color.b * 255.0) as u8,
        );

        let css = format!(
            "switch {{
                background-color: {off_hex};
                border-radius: 16px;
                min-width: 51px;
                min-height: 31px;
                padding: 0;
            }}
            switch:checked {{
                background-color: {on_hex};
            }}
            switch slider {{
                min-width: 27px;
                min-height: 27px;
                border-radius: 50%;
                background-color: white;
            }}",
        );
        crate::widget::apply_css(&switch, &css);

        if let Some(handler) = &self.on_change {
            let handler = handler.clone();
            switch.connect_state_set(move |_, is_on| {
                handler(is_on);
                glib::Propagation::Proceed
            });
        }

        // Wrap in a fixed-size box so the switch can't expand
        let wrapper = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        wrapper.set_hexpand(false);
        wrapper.set_width_request(51);
        wrapper.set_height_request(31);
        wrapper.set_valign(gtk::Align::Center);
        wrapper.append(&switch);
        wrapper.upcast()
    }
}

impl Default for Toggle {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewContent for Toggle {
    fn render(&self, _frame: Rect) -> gtk::Widget {
        self.build_switch()
    }

    fn can_become_first_responder(&self) -> bool {
        true
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        Size::new(51.0, 31.0)
    }
}

impl Widget for Toggle {
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
        self.build_switch()
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
    fn toggle_builder() {
        let toggle = Toggle::new().is_on(true);
        assert!(toggle.is_toggled());
    }

    #[test]
    fn toggle_default() {
        let toggle = Toggle::new();
        assert!(!toggle.is_toggled());
    }
}
