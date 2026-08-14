//! ScrollView widget — a scrollable container using GtkScrolledWindow.
//!
//! This is the UIKit equivalent of `UIScrollView` in Apple's UIKit.
//!
//! ```rust,no_run
//! use uikit::prelude::*;
//!
//! let mut content = View::empty();
//! content.set_frame(0.0, 0.0, 400.0, 800.0);
//!
//! let scroll = ScrollView::new()
//!     .content(content)
//!     .vertical(true)
//!     .horizontal(false);
//!
//! let view = View::new(scroll)
//!     .with_frame(0.0, 0.0, 400.0, 600.0);
//! ```

use crate::style::{Padding, Rect, Size};
use crate::view::{View, ViewContent};
use crate::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self, ScrolledWindow};

pub struct ScrollView {
    id: WidgetId,
    content: Option<View>,
    horizontal: bool,
    vertical: bool,
    position_mode: PositionMode,
    position: Position,
}

impl ScrollView {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            content: None,
            horizontal: false,
            vertical: true,
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

    pub fn content(mut self, content: View) -> Self {
        self.content = Some(content);
        self
    }

    pub fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }

    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    pub fn show_horizontal_scrollbar(mut self) -> Self {
        self.horizontal = true;
        self
    }

    pub fn show_vertical_scrollbar(mut self) -> Self {
        self.vertical = true;
        self
    }

    /// Create a View wrapping this ScrollView.
    pub fn to_view(self, x: f32, y: f32, width: f32, height: f32) -> View {
        View::new(self).with_frame(x, y, width, height)
    }
}

impl Default for ScrollView {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewContent for ScrollView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let scrolled = ScrolledWindow::new();

        scrolled.set_hscrollbar_policy(if self.horizontal {
            gtk::PolicyType::Automatic
        } else {
            gtk::PolicyType::Never
        });
        scrolled.set_vscrollbar_policy(if self.vertical {
            gtk::PolicyType::Automatic
        } else {
            gtk::PolicyType::Never
        });

        if let Some(ref content) = self.content {
            let child_widget = content.to_gtk();
            child_widget.set_hexpand(true);
            child_widget.set_vexpand(true);
            scrolled.set_child(Some(&child_widget));
        }

        if frame.width > 0.0 {
            scrolled.set_width_request(frame.width as i32);
        }
        if frame.height > 0.0 {
            scrolled.set_height_request(frame.height as i32);
        }

        scrolled.upcast()
    }

    fn size_that_fits(&self, available: Size) -> Size {
        available
    }
}

impl Widget for ScrollView {
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
        let scrolled = ScrolledWindow::new();

        scrolled.set_hscrollbar_policy(if self.horizontal {
            gtk::PolicyType::Automatic
        } else {
            gtk::PolicyType::Never
        });
        scrolled.set_vscrollbar_policy(if self.vertical {
            gtk::PolicyType::Automatic
        } else {
            gtk::PolicyType::Never
        });

        if let Some(ref content) = self.content {
            let child_widget = content.to_gtk();
            child_widget.set_hexpand(true);
            child_widget.set_vexpand(true);
            scrolled.set_child(Some(&child_widget));
        }

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
    fn scrollview_builder() {
        let scroll = ScrollView::new()
            .horizontal(true)
            .vertical(false);
        assert!(scroll.horizontal);
        assert!(!scroll.vertical);
    }

    #[test]
    fn scrollview_default() {
        let scroll = ScrollView::new();
        assert!(!scroll.horizontal);
        assert!(scroll.vertical);
    }
}
