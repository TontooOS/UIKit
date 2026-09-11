//! TrafficLights widget — macOS-style window controls (close, minimize, maximize).
//!
//! Shows colorful buttons when the window is active, gray when inactive.
//! Icons only appear on hover.

use crate::style::{Color, Padding};
use crate::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self, Box as GtkBox, Button as GtkButton, Image, Label, Orientation};
use std::rc::Rc;

/// Resolve a window-control icon at runtime.
///
/// Priority: `$UIKIT_ASSETS_DIR` override, LiveOS sidecar
/// (`/Library/System/uikit.resources/assets/`), staged sources
/// (`/Library/System/uikit/assets/`), then the crate dir (dev / `cargo run`).
/// Falls back to the LiveOS sidecar path when nothing exists so a missing
/// staging is obvious instead of a stale build path.
fn asset_path(file: &str) -> String {
    let mut candidates = Vec::new();
    if let Ok(env) = std::env::var("UIKIT_ASSETS_DIR") {
        if !env.is_empty() {
            candidates.push(format!("{}/{}", env.trim_end_matches('/'), file));
        }
    }
    candidates.push(format!("/Library/System/uikit.resources/assets/{}", file));
    candidates.push(format!("/Library/System/uikit/assets/{}", file));
    candidates.push(format!("{}/assets/{}", env!("CARGO_MANIFEST_DIR"), file));
    for c in &candidates {
        if std::path::Path::new(c).exists() {
            return c.clone();
        }
    }
    candidates.into_iter().next().unwrap_or_else(|| file.to_string())
}

pub struct TrafficLights {
    id: WidgetId,
    x: f32,
    y: f32,
    size: f32,
    spacing: f32,
    show_maximize: bool,
    title: String,
    /// Explicit bar height in px. None = auto (button size + 14 padding).
    bar_height: Option<f32>,
    /// Custom widget filling the bar after the traffic lights (the lights
    /// stay reserved on the left). Replaces the title zone when set.
    custom: Option<Rc<dyn Widget>>,
    /// Show the title text. Only renders when no custom widget is set.
    /// Default true.
    show_title: bool,
    /// Minimize button enabled. Default true. The backend disables it in
    /// fullscreen (stays gray, ignores clicks).
    minimize_enabled: bool,
}

impl TrafficLights {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            x: 18.0,
            y: 18.0,
            size: 17.0,
            spacing: 10.0,
            show_maximize: true,
            title: String::new(),
            bar_height: None,
            custom: None,
            show_title: true,
            minimize_enabled: true,
        }
    }

    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Show or hide the green maximize button (default: shown).
    pub fn show_maximize(mut self, show: bool) -> Self {
        self.show_maximize = show;
        self
    }

    /// Hide the green maximize button, keeping only close and minimize.
    pub fn without_maximize(self) -> Self {
        self.show_maximize(false)
    }

    /// Centered title shown in the middle of the title bar (empty = no title).
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Show or hide the title text (default: shown). Only renders when no
    /// custom widget is set.
    pub fn show_title(mut self, show: bool) -> Self {
        self.show_title = show;
        self
    }

    /// Hide the title text, keeping only lights (and custom content).
    pub fn without_title(self) -> Self {
        self.show_title(false)
    }

    /// Enable or disable the minimize (middle) button (default: enabled).
    /// Disabled stays gray and ignores clicks (used for fullscreen reveal).
    pub fn minimize_enabled(mut self, enabled: bool) -> Self {
        self.minimize_enabled = enabled;
        self
    }

    /// Custom widget filling the bar after the traffic lights, from the
    /// left (after the reserved lights) to the right edge. Replaces the
    /// title zone when set; the widget lays out its own alignment.
    pub fn with_custom(mut self, widget: impl Widget + 'static) -> Self {
        self.with_custom_shared(Rc::new(widget))
    }

    /// Shared-handle variant of [`with_custom`](Self::with_custom) for
    /// backends rendering the same content twice (windowed + fullscreen)
    /// or for wrappers (e.g. TontooUI `TitleBar`) holding shared content.
    pub fn with_custom_shared(mut self, widget: Rc<dyn Widget>) -> Self {
        self.custom = Some(widget);
        self
    }

    /// Explicit bar height in px (e.g. `WindowType::Mac` uses 44).
    /// Default None = auto (button size + 14 padding = 31).
    pub fn bar_height(mut self, height: f32) -> Self {
        self.bar_height = Some(height);
        self
    }

    fn build_button(
        &self,
        icon_path: &str,
        color: Color,
        on_click: impl Fn() + 'static,
    ) -> (GtkButton, Image) {
        let btn = GtkButton::new();
        btn.set_size_request(self.size as i32, self.size as i32);

        let color_hex = color.to_css();
        let size = self.size;

        let css = format!(
            "button.tl {{
                min-width: {size}px;
                min-height: {size}px;
                padding: 0;
                border-radius: 50%;
                background-color: #888888;
                background-image: none;
                border: none;
            }}
            button.tl-active {{
                min-width: {size}px;
                min-height: {size}px;
                padding: 0;
                border-radius: 50%;
                background-color: {color_hex};
                background-image: none;
                border: none;
            }}",
        );

        let provider = gtk::CssProvider::new();
        provider.load_from_string(&css);
        btn.style_context().add_provider(
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION as u32,
        );

        btn.add_css_class("tl");

        let icon = Image::from_file(icon_path);
        icon.set_pixel_size((size * 0.68) as i32);
        icon.set_visible(false);
        btn.set_child(Some(&icon));

        btn.connect_clicked(move |_| {
            on_click();
        });

        (btn, icon)
    }

    fn apply_focus_classes(buttons: &[GtkButton], active: bool) {
        for b in buttons {
            if active {
                b.add_css_class("tl-active");
            } else {
                b.remove_css_class("tl-active");
            }
        }
    }
}

impl Default for TrafficLights {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TrafficLights {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn position_mode(&self) -> PositionMode {
        PositionMode::Absolute
    }

    fn position(&self) -> Position {
        Position::new().at(self.x, self.y)
    }

    fn to_gtk(&self) -> gtk::Widget {
        let container = GtkBox::new(Orientation::Horizontal, self.spacing as i32);
        container.set_margin_start(self.x as i32);
        // Named so the backend can toggle the lights without touching the
        // rest of the bar (positions stay identical).
        container.set_widget_name("uikit-lights");
        // No top margin: vertical centering is handled by valign so the
        // lights sit exactly centered next to the title label.

        let close_path = asset_path("close.png");
        let minimize_path = asset_path("minimize.png");
        let maximize_path = asset_path("maximize.png");

        let (close_btn, close_icon) = self.build_button(&close_path, Color::from_rgb(255, 95, 86), || {
            crate::app::dispatch_custom("__close");
        });
        let minimize_click: fn() = if self.minimize_enabled {
            || crate::app::dispatch_custom("__minimize")
        } else {
            || {}
        };
        let (minimize_btn, minimize_icon) = self.build_button(&minimize_path, Color::from_rgb(255, 189, 46), minimize_click);

        container.append(&close_btn);
        container.append(&minimize_btn);

        // Disabled buttons stay out of the focus/hover sets, so they keep
        // the gray base style and show no icons.
        let mut icons = vec![close_icon];
        let mut buttons = vec![close_btn];
        if self.minimize_enabled {
            icons.push(minimize_icon);
            buttons.push(minimize_btn);
        }
        if self.show_maximize {
            let (maximize_btn, maximize_icon) = self.build_button(&maximize_path, Color::from_rgb(39, 201, 63), || {
                crate::app::dispatch_custom("__maximize");
            });
            container.append(&maximize_btn);
            icons.push(maximize_icon);
            buttons.push(maximize_btn);
        }
        let container_hover = container.clone();
        let motion = gtk::EventControllerMotion::new();
        {
            let icons = icons.clone();
            motion.connect_enter(move |_, _, _| {
                for icon in &icons {
                    icon.set_visible(true);
                }
            });
        }
        {
            motion.connect_leave(move |_| {
                for icon in &icons {
                    icon.set_visible(false);
                }
            });
        }
        container_hover.add_controller(motion);

        // Invisible drag handle so the window can be moved from the button bar.
        // Layout: lights left (always reserved), then either the app's
        // custom widget (fills the rest) or the centered title + spacer.
        let handle = gtk::WindowHandle::new();
        handle.set_hexpand(true);
        let row = GtkBox::new(Orientation::Horizontal, 0);
        row.set_hexpand(true);
        row.set_valign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);
        row.append(&container);
        if let Some(custom) = &self.custom {
            let widget = custom.to_gtk();
            widget.set_hexpand(true);
            widget.set_vexpand(true);
            widget.set_halign(gtk::Align::Fill);
            widget.set_valign(gtk::Align::Fill);
            row.append(&widget);
        } else if self.show_title && !self.title.is_empty() {
            let title = gtk::Label::new(Some(&self.title));
            title.set_hexpand(true);
            title.set_halign(gtk::Align::Center);
            title.set_valign(gtk::Align::Center);
            title.add_css_class("uikit-titlebar-title");
            row.append(&title);
            // Balances the lights block so the title stays truly centered.
            let spacer = GtkBox::new(Orientation::Horizontal, 0);
            spacer.set_size_request((self.x * 2.0 + self.size * 3.0 + self.spacing * 2.0) as i32, 1);
            spacer.set_valign(gtk::Align::Center);
            row.append(&spacer);
        }
        handle.set_child(Some(&row));
        // Height only: the background is controlled by the app-level
        // `.uikit-titlebar` rule (opaque scheme color) so the bar stays
        // fully solid while the content below can be transparent.
        let handle_css = format!(
            "windowhandle {{
                min-height: {bar_height}px;
            }}",
            bar_height = self.bar_height.unwrap_or(self.size + 14.0) as i32,
        );
        let handle_provider = gtk::CssProvider::new();
        handle_provider.load_from_string(&handle_css);
        handle.style_context().add_provider(
            &handle_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION as u32,
        );

        let container_weak = container.downgrade();
        // Connect the is-active watcher only once: `connect_map` fires on every
        // window map, so connecting inside it would accumulate an ever-growing
        // number of signal handlers across minimize/restore cycles.
        let connected = std::rc::Rc::new(std::cell::Cell::new(false));
        handle.connect_map(move |_| {
            if connected.get() {
                return;
            }
            connected.set(true);

            let Some(container) = container_weak.upgrade() else {
                return;
            };
            let Some(root) = container.root() else {
                return;
            };
            let Ok(window) = root.downcast::<gtk::Window>() else {
                return;
            };

            let buttons = buttons.clone();
            Self::apply_focus_classes(&buttons, window.is_active());

            window.connect_is_active_notify(move |w| {
                let active = w.is_active();
                Self::apply_focus_classes(&buttons, active);
            });
        });

        handle.upcast()
    }

    fn is_interactive(&self) -> bool {
        true
    }

    fn fill_width(&self) -> bool {
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
    fn traffic_lights_defaults() {
        let tl = TrafficLights::new();
        assert_eq!(tl.x, 18.0);
        assert_eq!(tl.y, 18.0);
        assert_eq!(tl.size, 17.0);
        assert_eq!(tl.spacing, 10.0);
    }

    #[test]
    fn traffic_lights_builder() {
        let tl = TrafficLights::new()
            .at(20.0, 20.0)
            .size(16.0)
            .spacing(10.0);
        assert_eq!(tl.x, 20.0);
        assert_eq!(tl.y, 20.0);
        assert_eq!(tl.size, 16.0);
        assert_eq!(tl.spacing, 10.0);
    }

    #[test]
    fn traffic_lights_maximize_toggle() {
        assert!(TrafficLights::new().show_maximize);
        assert!(!TrafficLights::new().without_maximize().show_maximize);
        assert!(!TrafficLights::new().show_maximize(false).show_maximize);
    }

    #[test]
    fn traffic_lights_title() {
        assert!(TrafficLights::new().title.is_empty());
        assert_eq!(TrafficLights::new().with_title("Volume Control").title, "Volume Control");
    }

    #[test]
    fn traffic_lights_bar_height() {
        assert!(TrafficLights::new().bar_height.is_none());
        assert_eq!(TrafficLights::new().bar_height(44.0).bar_height, Some(44.0));
    }

    #[test]
    fn traffic_lights_title_visibility() {
        assert!(TrafficLights::new().show_title);
        assert!(!TrafficLights::new().without_title().show_title);
        assert!(!TrafficLights::new().show_title(false).show_title);
    }

    #[test]
    fn traffic_lights_custom_slot() {
        use crate::widgets::Text;
        assert!(TrafficLights::new().custom.is_none());
        let bar = TrafficLights::new().with_custom(Text::new("tools"));
        assert!(bar.custom.is_some());
    }

    #[test]
    fn traffic_lights_minimize_enabled() {
        assert!(TrafficLights::new().minimize_enabled);
        assert!(!TrafficLights::new().minimize_enabled(false).minimize_enabled);
    }
}
