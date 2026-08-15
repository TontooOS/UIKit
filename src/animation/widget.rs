//! GTK widget binding.
//!
//! This is the bridge between the pure data [`Animator`] and real GTK4
//! widgets. [`AnimatedWidget`] maps a [`DynamicItem`] to a `gtk::Widget` and
//! applies the item's position, scale and opacity to the widget every frame.
//!
//! ```text
//! use uikit::animation::prelude::*;
//! // example: examples/animation.rs
//! ```

use uikitdynamics::animator::{Animator, DynamicItem};
use uikitdynamics::math::Vec2;
use uikitdynamics::math::Size;
use gtk::prelude::*;
use gtk::{self, Button, Overlay as GtkOverlay, Widget as GtkWidget};
use std::cell::RefCell;

// ═══════════════════════════════════════════════════════════════
// AnimatedWidget
// ═══════════════════════════════════════════════════════════════

/// A GTK widget driven by a physics item.
///
/// Each frame [`AnimatedWidget::apply`] reads the item state and writes it to
/// the widget: position becomes overlay margins (top-left = item center minus
/// half the size), opacity uses the native widget opacity, and scale is applied
/// through a per-widget CSS provider.
#[derive(Clone)]
pub struct AnimatedWidget {
    /// Index of the physics item that drives this widget.
    pub item_index: usize,
    /// Parent overlay the widget lives in.
    pub panel: GtkOverlay,
    /// The animated widget itself.
    pub widget: GtkWidget,
    /// Item size used to center the widget and drive collisions.
    pub size: Size,
    /// Cached CssProvider for scale transforms (avoids per-frame allocation).
    scale_provider: std::rc::Rc<RefCell<Option<gtk::CssProvider>>>,
}

impl AnimatedWidget {
    /// Create an animated widget from an existing physics item index.
    pub fn new(item_index: usize, widget: GtkWidget, panel: &GtkOverlay, size: Size) -> Self {
        panel.add_overlay(&widget);
        widget.set_halign(gtk::Align::Start);
        widget.set_valign(gtk::Align::Start);
        widget.set_visible(true);
        Self {
            item_index,
            panel: panel.clone(),
            widget,
            size,
            scale_provider: std::rc::Rc::new(RefCell::new(None)),
        }
    }

    /// Create a new physics item and bind `widget` to it.
    ///
    /// The item starts at the given center `position` and matches the given
    /// `size`. The item is woken (non-resting) so behaviors affect it.
    pub fn bind(
        animator: &mut Animator,
        widget: GtkWidget,
        panel: &GtkOverlay,
        position: Vec2,
        size: Size,
    ) -> Self {
        let mut item = DynamicItem::new(position, size.clone());
        item.wake();
        let index = animator.add_item(item);
        Self::new(index, widget, panel, size)
    }

    /// Create a GTK button bound to a new physics item.
    ///
    /// Returns the [`AnimatedWidget`] together with the underlying
    /// `gtk::Button` so callers can connect signal handlers.
    pub fn new_button(
        animator: &mut Animator,
        label: &str,
        panel: &GtkOverlay,
        position: Vec2,
        size: Size,
    ) -> (Self, Button) {
        let button = Button::with_label(label);
        let animated = Self::bind(animator, button.clone().upcast(), panel, position, size);
        (animated, button)
    }

    /// Apply the current physics item state to the widget.
    pub fn apply(&self, item: &DynamicItem) {
        let x = item.position.x - self.size.width / 2.0;
        let y = item.position.y - self.size.height / 2.0;
        self.widget.set_margin_start(x.max(0.0) as i32);
        self.widget.set_margin_top(y.max(0.0) as i32);

        self.widget.set_opacity(item.opacity.clamp(0.0, 1.0) as f64);

        let scale = item.scale;
        if (scale - 1.0).abs() > 0.001 {
            apply_scale_cached(&self.widget, scale, &self.scale_provider);
        }
        self.widget.queue_draw();
    }

    /// Drive the widget from a live animator (fetches the item by index).
    pub fn apply_from(&self, animator: &Animator) {
        if let Some(item) = animator.item(self.item_index) {
            self.apply(item);
        }
    }

    /// Immutable borrow of the current item from an animator.
    pub fn item<'a>(&self, animator: &'a Animator) -> Option<&'a DynamicItem> {
        animator.item(self.item_index)
    }

    /// Mutably borrow the current item from an animator.
    pub fn item_mut<'a>(&self, animator: &'a mut Animator) -> Option<&'a mut DynamicItem> {
        animator.item_mut(self.item_index)
    }

    /// Remove the widget from its overlay.
    pub fn remove(&self) {
        self.panel.remove_overlay(&self.widget);
    }

    /// Rectangle covered by the item (centered on `position`).
    pub fn bounds(&self, item: &DynamicItem) -> uikitdynamics::math::Rect {
        uikitdynamics::math::Rect::new(
            item.position.x - self.size.width / 2.0,
            item.position.y - self.size.height / 2.0,
            self.size.width,
            self.size.height,
        )
    }
}

// ═══════════════════════════════════════════════════════════════
// Scale helper
// ═══════════════════════════════════════════════════════════════

/// Apply a CSS scale transform to a widget, reusing a cached CssProvider.
fn apply_scale_cached(widget: &GtkWidget, scale: f32, cache: &std::rc::Rc<RefCell<Option<gtk::CssProvider>>>) {
    let css = format!(
        "* {{ transform: scale({scale}); transform-origin: center; }}"
    );
    let mut borrow = cache.borrow_mut();
    if let Some(ref provider) = *borrow {
        provider.load_from_string(&css);
    } else {
        let provider = gtk::CssProvider::new();
        provider.load_from_string(&css);
        widget.style_context().add_provider(
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION as u32,
        );
        *borrow = Some(provider);
    }
}