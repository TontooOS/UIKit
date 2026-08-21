//! Widget trait and core widget types for TontooUIKit.
//!
//! Every UI element in TontooUIKit implements the [`Widget`] trait.
//! Widgets produce GTK4 widgets via [`to_gtk()`](Widget::to_gtk).

use crate::style::{Color, Padding};
use gtk::prelude::*;
use gtk;

// ═══════════════════════════════════════════════════════════════
// WidgetId
// ═══════════════════════════════════════════════════════════════

pub type WidgetId = usize;

static mut NEXT_WIDGET_ID: WidgetId = 0;

pub fn next_widget_id() -> WidgetId {
    unsafe {
        let id = NEXT_WIDGET_ID;
        NEXT_WIDGET_ID += 1;
        id
    }
}

// ═══════════════════════════════════════════════════════════════
// PositionMode
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionMode {
    Auto,
    Absolute,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Position {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl Position {
    pub const fn new() -> Self {
        Self { x: None, y: None, width: None, height: None }
    }

    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.x = Some(x);
        self.y = Some(y);
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }
}

// ═══════════════════════════════════════════════════════════════
// Widget trait
// ═══════════════════════════════════════════════════════════════

/// The core trait that all widgets implement.
///
/// Each widget produces a GTK4 widget via [`to_gtk()`](Widget::to_gtk).
pub trait Widget {
    fn id(&self) -> WidgetId;

    fn children(&self) -> Vec<&dyn Widget> {
        Vec::new()
    }

    fn position_mode(&self) -> PositionMode {
        PositionMode::Auto
    }

    fn position(&self) -> Position {
        Position::new()
    }

    fn to_gtk(&self) -> gtk::Widget;

    fn is_interactive(&self) -> bool {
        false
    }

    fn flex_weight(&self) -> f32 {
        0.0
    }

    /// Whether this widget should fill the full container width.
    ///
    /// Layouts like `VStack` use this to `hexpand` the widget instead of
    /// left-aligning it, which is useful for full-width controls such as
    /// window drag bars.
    fn fill_width(&self) -> bool {
        false
    }

    /// Whether this widget should expand vertically to fill available height.
    ///
    /// `HStack` uses this to set `vexpand(true)` and `valign(Fill)` on the
    /// child, instead of centering it.
    fn expand_vertically(&self) -> bool {
        false
    }

    fn padding(&self) -> Padding {
        Padding::ZERO
    }
}

// ═══════════════════════════════════════════════════════════════
// WidgetNode — owning wrapper
// ═══════════════════════════════════════════════════════════════

pub struct WidgetNode {
    inner: Box<dyn Widget>,
}

impl WidgetNode {
    pub fn new(widget: impl Widget + 'static) -> Self {
        Self { inner: Box::new(widget) }
    }

    pub fn widget(&self) -> &dyn Widget {
        self.inner.as_ref()
    }

    pub fn into_inner(self) -> Box<dyn Widget> {
        self.inner
    }
}

impl Widget for WidgetNode {
    fn id(&self) -> WidgetId {
        self.inner.id()
    }

    fn children(&self) -> Vec<&dyn Widget> {
        self.inner.children()
    }

    fn position_mode(&self) -> PositionMode {
        self.inner.position_mode()
    }

    fn position(&self) -> Position {
        self.inner.position()
    }

    fn to_gtk(&self) -> gtk::Widget {
        self.inner.to_gtk()
    }

    fn is_interactive(&self) -> bool {
        self.inner.is_interactive()
    }

    fn flex_weight(&self) -> f32 {
        self.inner.flex_weight()
    }

    fn fill_width(&self) -> bool {
        self.inner.fill_width()
    }

    fn padding(&self) -> Padding {
        self.inner.padding()
    }
}

// ═══════════════════════════════════════════════════════════════
// CSS Helpers
// ═══════════════════════════════════════════════════════════════

/// Convert a Color to a CSS rgba() string.
pub fn color_to_css(c: Color) -> String {
    format!(
        "rgba({:.0}, {:.0}, {:.0}, {:.2})",
        c.r * 255.0,
        c.g * 255.0,
        c.b * 255.0,
        c.a
    )
}

/// Apply CSS inline to a GTK4 widget.
///
/// The provider is attached to the widget's own style context and cascades
/// to the widget's descendants (so class selectors targeting child widgets
/// such as `.sl-track` still work). Because the provider is scoped to the
/// widget it is released together with the widget when it is destroyed.
///
/// This intentionally does NOT register the provider on the display: doing so
/// would add a permanently retained, never-removable provider for every call,
/// which accumulates for the lifetime of the process and slows CSS matching
/// down as the app runs longer.
pub fn apply_css(widget: &impl IsA<gtk::Widget>, css: &str) {
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_string(css);
    widget.style_context().add_provider(
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION as u32,
    );
}
