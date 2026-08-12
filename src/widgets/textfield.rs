//! TextField widget — single-line text input using GtkEntry.

use crate::style::{Color, Padding};
use crate::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self, Entry};
use std::sync::Arc;

pub struct TextField {
    id: WidgetId,
    placeholder: String,
    text: String,
    show_clear: bool,
    password: bool,
    on_change: Option<Arc<dyn Fn(String) + Send + Sync>>,
    on_submit: Option<Arc<dyn Fn(String) + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl TextField {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            id: next_widget_id(),
            placeholder: placeholder.into(),
            text: String::new(),
            show_clear: false,
            password: false,
            on_change: None,
            on_submit: None,
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

    pub fn width(mut self, width: f32) -> Self {
        self.position.width = Some(width);
        self
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }

    pub fn on_submit(mut self, handler: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_submit = Some(Arc::new(handler));
        self
    }

    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    pub fn text_value(&self) -> &str {
        &self.text
    }
}

impl Widget for TextField {
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
        let entry = Entry::new();
        entry.set_placeholder_text(Some(&self.placeholder));
        if !self.text.is_empty() {
            entry.set_text(&self.text);
        }
        entry.set_visibility(!self.password);

        if let Some(w) = self.position.width {
            entry.set_width_request(w as i32);
        }

        let css = format!(
            "entry {{
                background-color: #2a2a2c;
                color: #ececec;
                border-radius: 8px;
                border: 1px solid #3a3a3d;
                padding: 8px 12px;
                font-family: 'SF Pro Display';
                font-size: 13px;
                caret-color: #0d8bff;
            }}
            entry:hover {{
                border-color: #4a4a4e;
            }}
            entry:focus {{
                border-color: #0d8bff;
            }}",
        );
        crate::widget::apply_css(&entry, &css);

        if let Some(handler) = &self.on_change {
            let handler = handler.clone();
            entry.connect_changed(move |e| {
                let value = e.text().to_string();
                handler(value);
            });
        }

        if let Some(handler) = &self.on_submit {
            let handler = handler.clone();
            entry.connect_activate(move |e| {
                let value = e.text().to_string();
                handler(value);
            });
        }

        if self.position_mode == PositionMode::Absolute {
            let mut css = String::from("entry {");
            if let Some(x) = self.position.x {
                css.push_str(&format!("margin-left: {}px;", x));
            }
            if let Some(y) = self.position.y {
                css.push_str(&format!("margin-top: {}px;", y));
            }
            css.push('}');
            crate::widget::apply_css(&entry, &css);
        }

        entry.upcast()
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
    fn textfield_builder() {
        let tf = TextField::new("Search...")
            .text("hello")
            .on_change(|_| {});
        assert_eq!(tf.placeholder(), "Search...");
        assert_eq!(tf.text_value(), "hello");
    }
}