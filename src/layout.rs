//! Layout system for TontooUIKit.
//!
//! - `VStack` → `GtkBox` (vertical)
//! - `HStack` → `GtkBox` (horizontal)
//! - `ZStack` → `GtkOverlay`
//! - `PaddingWrap` → `GtkBox` with margin
//! - `Frame` → `GtkBox` with fixed size

use crate::style::{HAlignment, Padding, VAlignment};
use crate::widget::{Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self, Orientation};

// ═══════════════════════════════════════════════════════════════
// VStack
// ═══════════════════════════════════════════════════════════════

pub struct VStack {
    id: WidgetId,
    children: Vec<Box<dyn Widget>>,
    spacing: f32,
    alignment: HAlignment,
}

impl VStack {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            children: Vec::new(),
            spacing: 0.0,
            alignment: HAlignment::Leading,
        }
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn alignment(mut self, alignment: HAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.children.push(Box::new(widget));
        self
    }
}

impl Default for VStack {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for VStack {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn children(&self) -> Vec<&dyn Widget> {
        self.children.iter().map(|c| c.as_ref()).collect()
    }

    fn to_gtk(&self) -> gtk::Widget {
        let vbox = gtk::Box::new(Orientation::Vertical, self.spacing as i32);
        vbox.set_hexpand(true);
        vbox.set_vexpand(true);

        let halign = match self.alignment {
            HAlignment::Leading => gtk::Align::Start,
            HAlignment::Center => gtk::Align::Center,
            HAlignment::Trailing => gtk::Align::End,
        };

        for child in &self.children {
            let gtk_child = child.to_gtk();

            // Flex children expand to fill available space.
            if child.flex_weight() > 0.0 {
                gtk_child.set_vexpand(true);
                gtk_child.set_valign(gtk::Align::Fill);
            } else if child.fill_width() {
                gtk_child.set_hexpand(true);
                gtk_child.set_halign(gtk::Align::Fill);
            } else {
                gtk_child.set_halign(halign);
            }

            // Apply child padding as margin.
            let pad = child.padding();
            if pad != Padding::ZERO {
                gtk_child.set_margin_top(pad.top as i32);
                gtk_child.set_margin_bottom(pad.bottom as i32);
                gtk_child.set_margin_start(pad.left as i32);
                gtk_child.set_margin_end(pad.right as i32);
            }

            vbox.append(&gtk_child);
        }

        vbox.upcast()
    }

    fn padding(&self) -> Padding {
        Padding::ZERO
    }
}

// ═══════════════════════════════════════════════════════════════
// HStack
// ═══════════════════════════════════════════════════════════════

pub struct HStack {
    id: WidgetId,
    children: Vec<Box<dyn Widget>>,
    spacing: f32,
    alignment: VAlignment,
}

impl HStack {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            children: Vec::new(),
            spacing: 0.0,
            alignment: VAlignment::Center,
        }
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn alignment(mut self, alignment: VAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.children.push(Box::new(widget));
        self
    }
}

impl Default for HStack {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for HStack {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn children(&self) -> Vec<&dyn Widget> {
        self.children.iter().map(|c| c.as_ref()).collect()
    }

    fn to_gtk(&self) -> gtk::Widget {
        let hbox = gtk::Box::new(Orientation::Horizontal, self.spacing as i32);
        hbox.set_hexpand(true);
        hbox.set_vexpand(true);

        let valign = match self.alignment {
            VAlignment::Top => gtk::Align::Start,
            VAlignment::Center => gtk::Align::Center,
            VAlignment::Bottom => gtk::Align::End,
        };

        for child in &self.children {
            let gtk_child = child.to_gtk();

            if child.flex_weight() > 0.0 {
                gtk_child.set_hexpand(true);
                gtk_child.set_halign(gtk::Align::Fill);
            } else {
                gtk_child.set_valign(valign);
            }

            let pad = child.padding();
            if pad != Padding::ZERO {
                gtk_child.set_margin_top(pad.top as i32);
                gtk_child.set_margin_bottom(pad.bottom as i32);
                gtk_child.set_margin_start(pad.left as i32);
                gtk_child.set_margin_end(pad.right as i32);
            }

            hbox.append(&gtk_child);
        }

        hbox.upcast()
    }

    fn padding(&self) -> Padding {
        Padding::ZERO
    }
}

// ═══════════════════════════════════════════════════════════════
// ZStack
// ═══════════════════════════════════════════════════════════════

pub struct ZStack {
    id: WidgetId,
    children: Vec<Box<dyn Widget>>,
}

impl ZStack {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            children: Vec::new(),
        }
    }

    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.children.push(Box::new(widget));
        self
    }
}

impl Default for ZStack {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ZStack {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn children(&self) -> Vec<&dyn Widget> {
        self.children.iter().map(|c| c.as_ref()).collect()
    }

    fn to_gtk(&self) -> gtk::Widget {
        let overlay = gtk::Overlay::new();

        for child in &self.children {
            let gtk_child = child.to_gtk();
            overlay.add_overlay(&gtk_child);
        }

        // Make first child the main widget for sizing.
        if let Some(first) = self.children.first() {
            let main = first.to_gtk();
            overlay.set_child(Some(&main));
        }

        overlay.upcast()
    }

    fn padding(&self) -> Padding {
        Padding::ZERO
    }
}

// ═══════════════════════════════════════════════════════════════
// PaddingWrap
// ═══════════════════════════════════════════════════════════════

pub struct PaddingWrap {
    id: WidgetId,
    child: Option<Box<dyn Widget>>,
    padding: Padding,
}

impl PaddingWrap {
    pub fn new(padding: Padding) -> Self {
        Self {
            id: next_widget_id(),
            child: None,
            padding,
        }
    }

    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(widget));
        self
    }
}

impl Widget for PaddingWrap {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn children(&self) -> Vec<&dyn Widget> {
        self.child.iter().map(|c| c.as_ref()).collect()
    }

    fn to_gtk(&self) -> gtk::Widget {
        let wrapper = gtk::Box::new(Orientation::Vertical, 0);
        wrapper.set_margin_top(self.padding.top as i32);
        wrapper.set_margin_bottom(self.padding.bottom as i32);
        wrapper.set_margin_start(self.padding.left as i32);
        wrapper.set_margin_end(self.padding.right as i32);

        if let Some(child) = &self.child {
            let gtk_child = child.to_gtk();
            wrapper.append(&gtk_child);
        }

        wrapper.upcast()
    }

    fn padding(&self) -> Padding {
        self.padding
    }
}

pub fn padding(padding: Padding) -> PaddingWrap {
    PaddingWrap::new(padding)
}

// ═══════════════════════════════════════════════════════════════
// Frame
// ═══════════════════════════════════════════════════════════════

pub struct Frame {
    id: WidgetId,
    child: Option<Box<dyn Widget>>,
    width: Option<f32>,
    height: Option<f32>,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            child: None,
            width: None,
            height: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(widget));
        self
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Frame {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn children(&self) -> Vec<&dyn Widget> {
        self.child.iter().map(|c| c.as_ref()).collect()
    }

    fn to_gtk(&self) -> gtk::Widget {
        let frame = gtk::Box::new(Orientation::Vertical, 0);

        if let Some(w) = self.width {
            frame.set_width_request(w as i32);
        }
        if let Some(h) = self.height {
            frame.set_height_request(h as i32);
        }

        if let Some(child) = &self.child {
            let gtk_child = child.to_gtk();
            frame.append(&gtk_child);
        }

        frame.upcast()
    }

    fn padding(&self) -> Padding {
        Padding::ZERO
    }
}

// Type alias
pub type PaddingLayout = PaddingWrap;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::Text;

    #[test]
    fn vstack_builder() {
        let stack = VStack::new()
            .spacing(8.0)
            .child(Text::new("A"))
            .child(Text::new("B"));
        assert_eq!(stack.children.len(), 2);
        assert_eq!(stack.spacing, 8.0);
    }

    #[test]
    fn hstack_builder() {
        let stack = HStack::new()
            .spacing(12.0)
            .alignment(VAlignment::Center)
            .child(Text::new("X"))
            .child(Text::new("Y"));
        assert_eq!(stack.children.len(), 2);
        assert_eq!(stack.alignment, VAlignment::Center);
    }

    #[test]
    fn zstack_builder() {
        let stack = ZStack::new()
            .child(Text::new("BG"))
            .child(Text::new("FG"));
        assert_eq!(stack.children.len(), 2);
    }

    #[test]
    fn padding_wrap() {
        let pw = PaddingWrap::new(Padding::all(16.0)).child(Text::new("Padded"));
        assert_eq!(pw.padding, Padding::all(16.0));
    }

    #[test]
    fn frame_builder() {
        let f = Frame::new().size(200.0, 100.0).child(Text::new("Framed"));
        assert_eq!(f.width, Some(200.0));
        assert_eq!(f.height, Some(100.0));
    }
}
