//! ListView widget — a scrollable list of items using GtkListView.
//!
//! This is the UIKit equivalent of `UITableView` in Apple's UIKit.
//!
//! ```rust,no_run
//! use uikit::prelude::*;
//!
//! let items = vec!["Item 1", "Item 2", "Item 3"];
//! let list = ListView::new()
//!     .items(items)
//!     .on_select(|index| println!("Selected: {}", index));
//!
//! let view = View::new(list)
//!     .with_frame(0.0, 0.0, 400.0, 600.0);
//! ```

use crate::style::{Color, Padding, Rect, Size};
use crate::view::{View, ViewContent};
use crate::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self, ScrolledWindow};
use std::sync::Arc;

pub struct ListView {
    id: WidgetId,
    items: Vec<String>,
    item_height: f32,
    show_dividers: bool,
    divider_color: Option<Color>,
    on_select: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl ListView {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            items: Vec::new(),
            item_height: 44.0,
            show_dividers: true,
            divider_color: None,
            on_select: None,
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

    pub fn items(mut self, items: Vec<impl Into<String>>) -> Self {
        self.items = items.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn add_item(mut self, item: impl Into<String>) -> Self {
        self.items.push(item.into());
        self
    }

    pub fn item_height(mut self, height: f32) -> Self {
        self.item_height = height;
        self
    }

    pub fn show_dividers(mut self, show: bool) -> Self {
        self.show_dividers = show;
        self
    }

    pub fn divider_color(mut self, color: Color) -> Self {
        self.divider_color = Some(color);
        self
    }

    pub fn on_select(mut self, handler: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(handler));
        self
    }

    /// Create a View wrapping this ListView.
    pub fn to_view(self, x: f32, y: f32, width: f32, height: f32) -> View {
        View::new(self).with_frame(x, y, width, height)
    }
}

impl Default for ListView {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewContent for ListView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        // Use a simple Box with labels for now (GtkListView requires more setup)
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);

        for (index, item) in self.items.iter().enumerate() {
            let item_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
            item_box.set_height_request(self.item_height as i32);

            let label = gtk::Label::new(Some(item));
            label.set_halign(gtk::Align::Start);
            label.set_valign(gtk::Align::Center);
            label.set_margin_start(16);
            label.set_margin_end(16);

            let css = format!(
                "label {{
                    font-family: 'SF Pro Display';
                    font-size: 13px;
                    color: #ececec;
                }}"
            );
            crate::widget::apply_css(&label, &css);

            item_box.append(&label);

            // Add divider if needed
            if self.show_dividers && index < self.items.len() - 1 {
                let divider = gtk::Separator::new(gtk::Orientation::Horizontal);
                let divider_color = self.divider_color
                    .unwrap_or(Color::from_hex("#3a3a3d").unwrap_or(Color::GRAY));
                let divider_css = format!(
                    "separator {{
                        min-height: 1px;
                        background-color: #{:02x}{:02x}{:02x};
                        margin-start: 16px;
                    }}",
                    (divider_color.r * 255.0) as u8,
                    (divider_color.g * 255.0) as u8,
                    (divider_color.b * 255.0) as u8,
                );
                crate::widget::apply_css(&divider, &divider_css);
                item_box.append(&divider);
            }

            // Add click handler
            if let Some(handler) = &self.on_select {
                let handler = handler.clone();
                let index = index;
                let gesture = gtk::GestureClick::new();
                gesture.connect_pressed(move |_, _, _, _| {
                    handler(index);
                });
                item_box.add_controller(gesture);
            }

            container.append(&item_box);
        }

        // Wrap in scrolled window
        let scrolled = ScrolledWindow::new();
        scrolled.set_child(Some(&container));
        scrolled.set_hscrollbar_policy(gtk::PolicyType::Never);
        scrolled.set_vscrollbar_policy(gtk::PolicyType::Automatic);
        crate::smooth_scroll::apply_smooth_scrolling(&scrolled);

        if frame.width > 0.0 {
            scrolled.set_width_request(frame.width as i32);
        }
        if frame.height > 0.0 {
            scrolled.set_height_request(frame.height as i32);
        }

        scrolled.upcast()
    }

    fn size_that_fits(&self, available: Size) -> Size {
        let total_height = self.items.len() as f32 * self.item_height;
        Size::new(available.width, total_height.min(available.height))
    }
}

impl Widget for ListView {
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
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);

        for (index, item) in self.items.iter().enumerate() {
            let item_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
            item_box.set_height_request(self.item_height as i32);

            let label = gtk::Label::new(Some(item));
            label.set_halign(gtk::Align::Start);
            label.set_valign(gtk::Align::Center);
            label.set_margin_start(16);
            label.set_margin_end(16);

            let css = format!(
                "label {{
                    font-family: 'SF Pro Display';
                    font-size: 13px;
                    color: #ececec;
                }}"
            );
            crate::widget::apply_css(&label, &css);

            item_box.append(&label);

            if self.show_dividers && index < self.items.len() - 1 {
                let divider = gtk::Separator::new(gtk::Orientation::Horizontal);
                let divider_color = self.divider_color
                    .unwrap_or(Color::from_hex("#3a3a3d").unwrap_or(Color::GRAY));
                let divider_css = format!(
                    "separator {{
                        min-height: 1px;
                        background-color: #{:02x}{:02x}{:02x};
                        margin-start: 16px;
                    }}",
                    (divider_color.r * 255.0) as u8,
                    (divider_color.g * 255.0) as u8,
                    (divider_color.b * 255.0) as u8,
                );
                crate::widget::apply_css(&divider, &divider_css);
                item_box.append(&divider);
            }

            if let Some(handler) = &self.on_select {
                let handler = handler.clone();
                let index = index;
                let gesture = gtk::GestureClick::new();
                gesture.connect_pressed(move |_, _, _, _| {
                    handler(index);
                });
                item_box.add_controller(gesture);
            }

            container.append(&item_box);
        }

        let scrolled = ScrolledWindow::new();
        scrolled.set_child(Some(&container));
        scrolled.set_hscrollbar_policy(gtk::PolicyType::Never);
        scrolled.set_vscrollbar_policy(gtk::PolicyType::Automatic);
        crate::smooth_scroll::apply_smooth_scrolling(&scrolled);

        scrolled.upcast()
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
    fn listview_builder() {
        let list = ListView::new()
            .items(vec!["A", "B", "C"])
            .item_height(60.0)
            .show_dividers(false);
        assert_eq!(list.items.len(), 3);
        assert_eq!(list.item_height, 60.0);
        assert!(!list.show_dividers);
    }

    #[test]
    fn listview_add_item() {
        let list = ListView::new()
            .add_item("Item 1")
            .add_item("Item 2");
        assert_eq!(list.items.len(), 2);
    }
}
