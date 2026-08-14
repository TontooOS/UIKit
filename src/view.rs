//! View — the base class for all UI elements in UIKit.
//!
//! `View` is the fundamental building block. Every visual element in
//! TontooOS UIKit is either a `View` or wraps one. Custom views are
//! created by implementing [`ViewContent`] and providing a GTK4 rendering.
//!
//! ```rust,no_run
//! use uikit::prelude::*;
//!
//! struct MyCustomView;
//!
//! impl ViewContent for MyCustomView {
//!     fn render(&self, frame: Rect) -> gtk::Widget {
//!         let label = gtk::Label::new(Some("Custom View"));
//!         label.upcast()
//!     }
//! }
//!
//! let view = View::new(MyCustomView)
//!     .frame(0.0, 0.0, 200.0, 100.0)
//!     .background(Color::from_hex("#2a2a2c").unwrap());
//! ```

use crate::style::{Color, Rect};
use crate::widget::next_widget_id;
use gtk::prelude::*;

// ═══════════════════════════════════════════════════════════════
// ViewId
// ═══════════════════════════════════════════════════════════════

pub type ViewId = usize;

// ═══════════════════════════════════════════════════════════════
// ViewContent trait
// ═══════════════════════════════════════════════════════════════

/// Trait for custom view content.
///
/// Implement this to create custom views. The `render` method produces
/// the GTK4 widget that represents this view.
pub trait ViewContent {
    /// Render this view's content as a GTK4 widget.
    fn render(&self, frame: Rect) -> gtk::Widget;

    /// Called when the view's frame changes.
    fn layout(&mut self, _frame: Rect) {}

    /// Called after the view is added to a superview.
    fn did_move_to_superview(&mut self) {}

    /// Called when the view is about to be removed from its superview.
    fn will_move_from_superview(&mut self) {}

    /// Called when the view appears on screen.
    fn did_appear(&mut self) {}

    /// Called when the view disappears from screen.
    fn will_disappear(&mut self) {}

    /// Returns the preferred size for this view given available space.
    fn size_that_fits(&self, available: crate::style::Size) -> crate::style::Size {
        available
    }

    /// Whether this view can become the first responder.
    fn can_become_first_responder(&self) -> bool {
        false
    }
}

// ═══════════════════════════════════════════════════════════════
// View
// ═══════════════════════════════════════════════════════════════

/// The base view class for UIKit.
///
/// Every visual element in UIKit is represented as a `View`. Views form a
/// tree structure with parent-child relationships (superview/subviews).
///
/// # Layout
///
/// Views use a constraint-based layout system. You add constraints to define
/// how views are positioned relative to each other or their superview.
///
/// # Custom Views
///
/// To create a custom view, implement [`ViewContent`] and wrap it in a `View`:
///
/// ```rust,no_run
/// use uikit::prelude::*;
///
/// struct CardContent;
///
/// impl ViewContent for CardContent {
///     fn render(&self, frame: Rect) -> gtk::Widget {
///         let card = gtk::Box::new(gtk::Orientation::Vertical, 8);
///         card.add_css_class("glass-panel");
///         card.upcast()
///     }
/// }
///
/// let card = View::new(CardContent)
///     .frame(16.0, 16.0, 300.0, 200.0)
///     .background(Color::TRANSPARENT);
/// ```
pub struct View {
    id: ViewId,
    frame: Rect,
    bounds: Rect,
    background_color: Option<Color>,
    is_hidden: bool,
    alpha: f32,
    tag: i32,
    content: Option<Box<dyn ViewContent>>,
    subviews: Vec<View>,
    superview: Option<ViewId>,
    needs_layout: bool,
}

impl View {
    /// Create a new view with the given content.
    pub fn new(content: impl ViewContent + 'static) -> Self {
        Self {
            id: next_widget_id(),
            frame: Rect::ZERO,
            bounds: Rect::ZERO,
            background_color: None,
            is_hidden: false,
            alpha: 1.0,
            tag: 0,
            content: Some(Box::new(content)),
            subviews: Vec::new(),
            superview: None,
            needs_layout: true,
        }
    }

    /// Create an empty view (no content, used as container).
    pub fn empty() -> Self {
        Self {
            id: next_widget_id(),
            frame: Rect::ZERO,
            bounds: Rect::ZERO,
            background_color: None,
            is_hidden: false,
            alpha: 1.0,
            tag: 0,
            content: None,
            subviews: Vec::new(),
            superview: None,
            needs_layout: true,
        }
    }

    /// Create a view from a GTK4 widget directly.
    pub fn from_gtk(widget: gtk::Widget) -> Self {
        struct GtkContent(gtk::Widget);
        impl ViewContent for GtkContent {
            fn render(&self, _frame: Rect) -> gtk::Widget {
                self.0.clone().upcast()
            }
        }
        Self::new(GtkContent(widget))
    }

    // ─── Frame & Bounds ────────────────────────────────────

    pub fn id(&self) -> ViewId {
        self.id
    }

    pub fn frame(&self) -> Rect {
        self.frame
    }

    pub fn set_frame(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.frame = Rect::new(x, y, width, height);
        self.bounds = Rect::new(0.0, 0.0, width, height);
        self.needs_layout = true;
    }

    pub fn with_frame(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        self.set_frame(x, y, width, height);
        self
    }

    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    pub fn set_bounds(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.bounds = Rect::new(x, y, width, height);
    }

    pub fn center(&self) -> crate::style::Point {
        crate::style::Point::new(
            self.frame.x + self.frame.width / 2.0,
            self.frame.y + self.frame.height / 2.0,
        )
    }

    pub fn set_center(&mut self, x: f32, y: f32) {
        self.frame.x = x - self.frame.width / 2.0;
        self.frame.y = y - self.frame.height / 2.0;
    }

    pub fn width(&self) -> f32 {
        self.frame.width
    }

    pub fn height(&self) -> f32 {
        self.frame.height
    }

    pub fn set_width(&mut self, width: f32) {
        self.frame.width = width;
        self.bounds.width = width;
        self.needs_layout = true;
    }

    pub fn set_height(&mut self, height: f32) {
        self.frame.height = height;
        self.bounds.height = height;
        self.needs_layout = true;
    }

    // ─── Appearance ────────────────────────────────────────

    pub fn background_color(&self) -> Option<Color> {
        self.background_color
    }

    pub fn set_background_color(&mut self, color: Option<Color>) {
        self.background_color = color;
    }

    pub fn with_background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn is_hidden(&self) -> bool {
        self.is_hidden
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.is_hidden = hidden;
    }

    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha.clamp(0.0, 1.0);
    }

    pub fn tag(&self) -> i32 {
        self.tag
    }

    pub fn set_tag(&mut self, tag: i32) {
        self.tag = tag;
    }

    // ─── View Hierarchy ────────────────────────────────────

    pub fn superview(&self) -> Option<ViewId> {
        self.superview
    }

    pub fn subviews(&self) -> &[View] {
        &self.subviews
    }

    pub fn subviews_mut(&mut self) -> &mut Vec<View> {
        &mut self.subviews
    }

    /// Add a subview to this view.
    pub fn add_subview(&mut self, mut subview: View) {
        subview.superview = Some(self.id);
        subview.needs_layout = true;
        if let Some(ref mut content) = subview.content {
            content.did_move_to_superview();
        }
        self.subviews.push(subview);
    }

    /// Remove a subview by its id.
    pub fn remove_subview(&mut self, view_id: ViewId) -> Option<View> {
        if let Some(pos) = self.subviews.iter().position(|v| v.id == view_id) {
            let mut removed = self.subviews.remove(pos);
            removed.superview = None;
            if let Some(ref mut content) = removed.content {
                content.will_move_from_superview();
            }
            Some(removed)
        } else {
            None
        }
    }

    /// Remove all subviews.
    pub fn remove_all_subviews(&mut self) {
        for mut subview in self.subviews.drain(..) {
            subview.superview = None;
            if let Some(ref mut content) = subview.content {
                content.will_move_from_superview();
            }
        }
    }

    /// Find a subview by id (recursive).
    pub fn view_by_id(&self, view_id: ViewId) -> Option<&View> {
        if self.id == view_id {
            return Some(self);
        }
        for subview in &self.subviews {
            if let Some(found) = subview.view_by_id(view_id) {
                return Some(found);
            }
        }
        None
    }

    /// Find a subview by id (recursive, mutable).
    pub fn view_by_id_mut(&mut self, view_id: ViewId) -> Option<&mut View> {
        if self.id == view_id {
            return Some(self);
        }
        for subview in &mut self.subviews {
            if let Some(found) = subview.view_by_id_mut(view_id) {
                return Some(found);
            }
        }
        None
    }

    /// Find a subview by tag (recursive).
    pub fn view_by_tag(&self, tag: i32) -> Option<&View> {
        if self.tag == tag {
            return Some(self);
        }
        for subview in &self.subviews {
            if let Some(found) = subview.view_by_tag(tag) {
                return Some(found);
            }
        }
        None }
    /// Find the first subview with a given tag (recursive, mutable).
    pub fn view_by_tag_mut(&mut self, tag: i32) -> Option<&mut View> {
        if self.tag == tag {
            return Some(self);
        }
        for subview in &mut self.subviews {
            if let Some(found) = subview.view_by_tag_mut(tag) {
                return Some(found);
            }
        }
        None
    }

    /// Bring a subview to the front (move to end of subviews array).
    pub fn bringSubviewToFront(&mut self, view_id: ViewId) {
        if let Some(pos) = self.subviews.iter().position(|v| v.id == view_id) {
            let view = self.subviews.remove(pos);
            self.subviews.push(view);
        }
    }

    /// Send a subview to the back (move to beginning of subviews array).
    pub fn sendSubviewToBack(&mut self, view_id: ViewId) {
        if let Some(pos) = self.subviews.iter().position(|v| v.id == view_id) {
            let view = self.subviews.remove(pos);
            self.subviews.insert(0, view);
        }
    }

    // ─── Layout ────────────────────────────────────────────

    /// Mark this view as needing layout.
    pub fn set_needs_layout(&mut self) {
        self.needs_layout = true;
    }

    /// Force layout of this view and all subviews.
    pub fn layout_if_needed(&mut self) {
        if self.needs_layout {
            self.layoutSubviews();
            self.needs_layout = false;
        }
        for subview in &mut self.subviews {
            subview.layout_if_needed();
        }
    }

    /// Override this in subclasses to layout subviews.
    pub fn layoutSubviews(&mut self) {
        if let Some(ref mut content) = self.content {
            content.layout(self.bounds);
        }
        // Notify subviews of layout
        for subview in &mut self.subviews {
            subview.layoutSubviews();
        }
    }

    // ─── Hit Testing ───────────────────────────────────────

    /// Determine which view is hit at the given point (in this view's coordinate system).
    pub fn hit_test(&self, point: crate::style::Point) -> Option<&View> {
        if self.is_hidden || self.alpha <= 0.01 {
            return None;
        }

        // Check if point is within this view's bounds
        if point.x < 0.0 || point.x > self.bounds.width
            || point.y < 0.0 || point.y > self.bounds.height
        {
            return None;
        }

        // Check subviews in reverse order (topmost first)
        for subview in self.subviews.iter().rev() {
            // Convert point to subview's coordinate system
            let local_point = crate::style::Point::new(
                point.x - subview.frame.x + self.bounds.x,
                point.y - subview.frame.y + self.bounds.y,
            );
            if let Some(hit) = subview.hit_test(local_point) {
                return Some(hit);
            }
        }

        // No subview hit, so this view is the hit view
        Some(self)
    }

    // ─── Rendering ─────────────────────────────────────────

    /// Render this view and all subviews to a GTK4 widget.
    pub fn to_gtk(&self) -> gtk::Widget {
        self.to_gtk_inner(false)
    }

    /// Render without fixed height constraints (for scroll containers).
    pub fn to_gtk_scrollable(&self) -> gtk::Widget {
        self.to_gtk_inner(true)
    }

    fn to_gtk_inner(&self, scrollable: bool) -> gtk::Widget {
        // For scrollable views, use a vertical Box so children stack and expand naturally.
        if scrollable {
            let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);
            vbox.set_hexpand(true);
            vbox.set_vexpand(false);
            vbox.set_margin_top(8);
            vbox.set_margin_bottom(8);

            if let Some(ref content) = self.content {
                let widget = content.render(self.bounds);
                widget.set_hexpand(true);
                if let Some(bg) = self.background_color {
                    let css = format!("* {{ background-color: {}; }}", bg.to_css());
                    crate::widget::apply_css(&widget, &css);
                }
                if self.alpha < 1.0 { widget.set_opacity(self.alpha as f64); }
                widget.set_visible(!self.is_hidden);
                vbox.append(&widget);
            }

            for subview in &self.subviews {
                let sub_widget = subview.to_gtk_inner(true);
                sub_widget.set_hexpand(true);
                sub_widget.set_margin_start(subview.frame.x as i32);
                // Don't use frame.y for scrollable — the Box stacks naturally.
                // Only apply margin_top if it's a small gap (not an absolute position).
                if subview.frame.y > 0.0 && subview.frame.y <= 4.0 {
                    sub_widget.set_margin_top(subview.frame.y as i32);
                }
                if subview.frame.width > 0.0 {
                    sub_widget.set_width_request(subview.frame.width as i32);
                }
                vbox.append(&sub_widget);
            }

            return vbox.upcast();
        }

        // Standard non-scrollable rendering with Overlay
        let container = gtk::Overlay::new();
        container.set_hexpand(true);
        container.set_vexpand(true);

        // Render content
        if let Some(ref content) = self.content {
            let widget = content.render(self.bounds);
            widget.set_hexpand(true);
            widget.set_vexpand(true);

            // Apply frame size
            if self.frame.width > 0.0 {
                widget.set_width_request(self.frame.width as i32);
            }
            if self.frame.height > 0.0 {
                widget.set_height_request(self.frame.height as i32);
            }

            // Apply background color
            if let Some(bg) = self.background_color {
                let css = format!(
                    "* {{ background-color: {}; border-radius: 8px; }}",
                    bg.to_css()
                );
                crate::widget::apply_css(&widget, &css);
            }

            // Apply alpha
            if self.alpha < 1.0 {
                widget.set_opacity(self.alpha as f64);
            }

            // Apply visibility
            widget.set_visible(!self.is_hidden);

            container.set_child(Some(&widget));
        }

        // Render subviews
        for subview in &self.subviews {
            let sub_widget = subview.to_gtk();
            sub_widget.set_hexpand(false);
            sub_widget.set_vexpand(false);

            // Position subview
            sub_widget.set_margin_start(subview.frame.x as i32);
            sub_widget.set_margin_top(subview.frame.y as i32);

            if subview.frame.width > 0.0 {
                sub_widget.set_width_request(subview.frame.width as i32);
            }
            if subview.frame.height > 0.0 {
                sub_widget.set_height_request(subview.frame.height as i32);
            }

            container.add_overlay(&sub_widget);
        }

        container.upcast()
    }
}

impl Default for View {
    fn default() -> Self {
        Self::empty()
    }
}

// ═══════════════════════════════════════════════════════════════
// LayoutConstraint
// ═══════════════════════════════════════════════════════════════

/// A layout constraint between two anchors.
#[derive(Debug, Clone)]
pub struct LayoutConstraint {
    pub first_item: ViewId,
    pub first_attribute: Anchor,
    pub relation: ConstraintRelation,
    pub second_item: ViewId,
    pub second_attribute: Anchor,
    pub multiplier: f32,
    pub constant: f32,
    pub priority: f32,
}

/// Anchor points for constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    Top,
    Bottom,
    Leading,
    Trailing,
    CenterX,
    CenterY,
    Width,
    Height,
}

/// Constraint relation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintRelation {
    Equal,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

impl LayoutConstraint {
    pub fn new(
        first_item: ViewId,
        first_attribute: Anchor,
        relation: ConstraintRelation,
        second_item: ViewId,
        second_attribute: Anchor,
        multiplier: f32,
        constant: f32,
    ) -> Self {
        Self {
            first_item,
            first_attribute,
            relation,
            second_item,
            second_attribute,
            multiplier,
            constant,
            priority: 1000.0, // Required priority by default
        }
    }

    pub fn with_priority(mut self, priority: f32) -> Self {
        self.priority = priority;
        self
    }
}

// ═══════════════════════════════════════════════════════════════
// ViewBuilder — convenience builder for Views
// ═══════════════════════════════════════════════════════════════

/// Builder for creating Views with a fluent API.
pub struct ViewBuilder {
    view: View,
}

impl ViewBuilder {
    pub fn new(content: impl ViewContent + 'static) -> Self {
        Self {
            view: View::new(content),
        }
    }

    pub fn empty() -> Self {
        Self {
            view: View::empty(),
        }
    }

    pub fn frame(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        self.view.set_frame(x, y, width, height);
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.view.set_background_color(Some(color));
        self
    }

    pub fn tag(mut self, tag: i32) -> Self {
        self.view.set_tag(tag);
        self
    }

    pub fn alpha(mut self, alpha: f32) -> Self {
        self.view.set_alpha(alpha);
        self
    }

    pub fn hidden(mut self, hidden: bool) -> Self {
        self.view.set_hidden(hidden);
        self
    }

    pub fn subview(mut self, subview: View) -> Self {
        self.view.add_subview(subview);
        self
    }

    pub fn build(self) -> View {
        self.view
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Point;

    struct TestContent;

    impl ViewContent for TestContent {
        fn render(&self, _frame: Rect) -> gtk::Widget {
            gtk::Label::new(Some("Test")).upcast()
        }
    }

    #[test]
    fn view_creation() {
        let view = View::new(TestContent);
        assert_eq!(view.width(), 0.0);
        assert_eq!(view.height(), 0.0);
        assert!(!view.is_hidden());
        assert_eq!(view.alpha(), 1.0);
    }

    #[test]
    fn view_frame() {
        let view = View::new(TestContent).with_frame(10.0, 20.0, 100.0, 50.0);
        assert_eq!(view.frame().x, 10.0);
        assert_eq!(view.frame().y, 20.0);
        assert_eq!(view.width(), 100.0);
        assert_eq!(view.height(), 50.0);
    }

    #[test]
    fn view_subviews() {
        let mut parent = View::empty();
        let child1 = View::new(TestContent).with_tag(1);
        let child2 = View::new(TestContent).with_tag(2);

        parent.add_subview(child1);
        parent.add_subview(child2);

        assert_eq!(parent.subviews().len(), 2);
        assert!(parent.view_by_tag(1).is_some());
        assert!(parent.view_by_tag(2).is_some());
    }

    #[test]
    fn view_hit_test() {
        let mut parent = View::empty();
        parent.set_frame(0.0, 0.0, 200.0, 200.0);

        let mut child = View::new(TestContent);
        child.set_frame(10.0, 10.0, 50.0, 50.0);
        child.set_tag(42);
        parent.add_subview(child);

        // Point inside child
        let hit = parent.hit_test(Point::new(30.0, 30.0));
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().tag(), 42);

        // Point outside child
        let miss = parent.hit_test(Point::new(100.0, 100.0));
        assert!(miss.is_some()); // hits parent
        assert_eq!(miss.unwrap().tag(), 0);
    }

    #[test]
    fn view_builder() {
        let view = ViewBuilder::new(TestContent)
            .frame(0.0, 0.0, 100.0, 100.0)
            .background(Color::RED)
            .tag(5)
            .alpha(0.8)
            .build();

        assert_eq!(view.width(), 100.0);
        assert_eq!(view.tag(), 5);
        assert_eq!(view.alpha(), 0.8);
        assert!(view.background_color().is_some());
    }
}
