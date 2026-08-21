//! TrafficLights widget — macOS-style window controls (close, minimize, maximize).
//!
//! Shows colorful buttons when the window is active, gray when inactive.
//! Icons only appear on hover.

use crate::style::{Color, Padding};
use crate::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self, Box as GtkBox, Button as GtkButton, Image, Orientation};

pub struct TrafficLights {
    id: WidgetId,
    x: f32,
    y: f32,
    size: f32,
    spacing: f32,
}

impl TrafficLights {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            x: 18.0,
            y: 18.0,
            size: 17.0,
            spacing: 10.0,
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
        container.set_margin_top(self.y as i32);

        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let close_path = format!("{}/assets/close.png", manifest_dir);
        let minimize_path = format!("{}/assets/minimize.png", manifest_dir);
        let maximize_path = format!("{}/assets/maximize.png", manifest_dir);

        let (close_btn, close_icon) = self.build_button(&close_path, Color::from_rgb(255, 95, 86), || {
            crate::app::dispatch_custom("__close");
        });
        let (minimize_btn, minimize_icon) = self.build_button(&minimize_path, Color::from_rgb(255, 189, 46), || {
            crate::app::dispatch_custom("__minimize");
        });
        let (maximize_btn, maximize_icon) = self.build_button(&maximize_path, Color::from_rgb(39, 201, 63), || {
            crate::app::dispatch_custom("__maximize");
        });

        container.append(&close_btn);
        container.append(&minimize_btn);
        container.append(&maximize_btn);

        let icons = vec![close_icon, minimize_icon, maximize_icon];
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
        let handle = gtk::WindowHandle::new();
        handle.set_hexpand(true);
        container.set_valign(gtk::Align::Center);
        handle.set_child(Some(&container));
        let handle_css = format!(
            "windowhandle {{ 
                min-height: {bar_height}px;
                background: transparent;
            }}",
            bar_height = (self.size + 14.0) as i32,
        );
        let handle_provider = gtk::CssProvider::new();
        handle_provider.load_from_string(&handle_css);
        handle.style_context().add_provider(
            &handle_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION as u32,
        );

        let buttons = vec![close_btn, minimize_btn, maximize_btn];
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
}
