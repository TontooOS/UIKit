//! Application runtime for TontooUIKit.
//!
//! Uses `gtk::Application` for the event loop and window management.
//! Delegate-driven state updates with automatic view rebuilding.

use crate::widget::Widget;
use gtk::prelude::*;
use gtk::{self, Application, ApplicationWindow};
use std::cell::RefCell;
use std::time::Duration;

/// Color scheme preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
    Dark = 0,
    Light = 1,
}

impl Default for ColorScheme {
    fn default() -> Self { Self::Dark }
}

/// Application delegate trait — implement this for your app.
pub trait AppDelegate: Send + 'static {
    fn view(&self) -> Box<dyn Widget>;
    fn handle_custom(&mut self, _action: &str) {}
}

/// No-op delegate.
pub struct NoDelegate;

impl AppDelegate for NoDelegate {
    fn view(&self) -> Box<dyn Widget> {
        Box::new(crate::widgets::Text::new(""))
    }
}

/// Global app state for dispatching custom actions.
struct AppState {
    delegate: Box<dyn AppDelegate>,
    window: Option<gtk::ApplicationWindow>,
}

thread_local! {
    static APP_STATE: RefCell<Option<AppState>> = RefCell::new(None);
}

/// Called by buttons when a custom action fires.
pub fn dispatch_custom(action_name: &str) {
    APP_STATE.with(|state| {
        let mut state = state.borrow_mut();
        if let Some(ref mut app) = *state {
            // Handle built-in window actions
            match action_name {
                "__close" => {
                    if let Some(ref window) = app.window {
                        window.close();
                    }
                    return;
                }
                "__minimize" => {
                    if let Some(ref window) = app.window {
                        window.minimize();
                    }
                    return;
                }
                "__maximize" => {
                    if let Some(ref window) = app.window {
                        if window.is_maximized() {
                            window.unmaximize();
                        } else {
                            window.maximize();
                        }
                    }
                    return;
                }
                _ => {}
            }

            // Delegate custom actions
            app.delegate.handle_custom(action_name);
            let view = app.delegate.view();
            let gtk_widget = wrap_with_window_bar(view);
            gtk_widget.set_hexpand(true);
            gtk_widget.set_vexpand(true);
            gtk_widget.set_visible(true);
            if let Some(ref window) = app.window {
                window.set_child(Some(&gtk_widget));
                window.queue_draw();
            }
        }
    });
}

/// Wrap an app view with the standard UIKIT window chrome:
/// a full-width drag bar with the macOS-style traffic lights on top.
fn wrap_with_window_bar(view: Box<dyn Widget>) -> gtk::Widget {
    let bar = crate::widgets::TrafficLights::new();
    let bar_widget = bar.to_gtk();
    bar_widget.set_hexpand(true);

    let view_widget = view.to_gtk();
    view_widget.set_hexpand(true);
    view_widget.set_vexpand(true);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.set_hexpand(true);
    vbox.set_vexpand(true);
    vbox.append(&bar_widget);
    vbox.append(&view_widget);
    vbox.upcast()
}

/// The top-level application container.
pub struct App {
    title: String,
    width: i32,
    height: i32,
    root: Option<Box<dyn Widget>>,
    color_scheme: ColorScheme,
    glass: Option<(f32, f32, f32)>,
    delegate: Option<Box<dyn AppDelegate>>,
    /// Optional per-frame callback (dt in seconds) driven by a 60 Hz timer.
    tick: Option<Box<dyn FnMut(f32) + Send>>,
}

impl App {
    pub fn new(title: impl Into<String>, width: i32, height: i32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
            root: None,
            color_scheme: ColorScheme::Dark,
            glass: None,
            delegate: None,
            tick: None,
        }
    }

    pub fn with_delegate(
        title: impl Into<String>,
        width: i32,
        height: i32,
        delegate: impl AppDelegate,
    ) -> Self {
        let mut app = Self::new(title, width, height);
        app.delegate = Some(Box::new(delegate));
        app
    }

    pub fn set_root(&mut self, widget: impl Widget + 'static) {
        self.root = Some(Box::new(widget));
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    pub fn set_size(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }

    pub fn set_color_scheme(&mut self, scheme: ColorScheme) {
        self.color_scheme = scheme;
    }

    pub fn set_glass(&mut self, milkiness: f32, alpha: f32, sigma: f32) {
        self.glass = Some((milkiness, alpha, sigma));
    }

    pub fn clear_glass(&mut self) {
        self.glass = None;
    }

    /// Register a per-frame callback invoked ~60 times per second.
    ///
    /// The callback receives the frame delta `dt` in seconds. This is the
    /// standard driver for [`Animator`](crate::animation::Animator) ticks:
    ///
    /// ```no_run
    /// # use tontoo_uikit::prelude::*;
    /// # use tontoo_uikit::animation::*;
    /// # use tontoo_uikit::style::Size;
    /// # let mut app = App::new("Animation", 800, 600);
    /// let mut animator = Animator::new();
    /// animator.set_bounds(Some(Rect::new(0.0, 0.0, 800.0, 600.0)));
    /// app.set_tick(move |dt| animator.tick(dt));
    /// ```
    pub fn set_tick(&mut self, tick: impl FnMut(f32) + Send + 'static) {
        self.tick = Some(Box::new(tick));
    }

    pub fn title(&self) -> &str { &self.title }
    pub fn width(&self) -> i32 { self.width }
    pub fn height(&self) -> i32 { self.height }
    pub fn color_scheme(&self) -> ColorScheme { self.color_scheme }
    pub fn glass(&self) -> Option<(f32, f32, f32)> { self.glass }

    /// Fix WSLg rendering issues by linking Wayland sockets and setting
    /// environment variables for software rendering.
    fn fix_wslg_environment() {
        // Only on Linux/WSL
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            use std::path::Path;

            // Try to link Wayland socket if missing (systemd wipes it).
            if let Ok(uid) = fs::read_to_string("/proc/self/loginuid").or_else(|_| fs::read_to_string("/proc/sys/kernel/osrelease").map(|_| "0".to_string())) {
                let uid_str = uid.trim();
                if uid_str != "0" && uid_str != "4294967295" {
                    if let Ok(uid_num) = uid_str.parse::<u32>() {
                        let runtime_dir = format!("/run/user/{}", uid_num);
                        let wayland_socket = format!("{}/wayland-0", runtime_dir);
                        let wslg_socket = "/mnt/wslg/runtime-dir/wayland-0";

                        if !Path::new(&wayland_socket).exists() && Path::new(wslg_socket).exists() {
                            let _ = fs::create_dir_all(&runtime_dir);
                            let _ = fs::remove_file(&wayland_socket);
                            let _ = std::os::unix::fs::symlink(wslg_socket, &wayland_socket);
                            let lock_src = "/mnt/wslg/runtime-dir/wayland-0.lock";
                            let lock_dst = format!("{}/wayland-0.lock", runtime_dir);
                            let _ = fs::remove_file(&lock_dst);
                            let _ = std::os::unix::fs::symlink(lock_src, &lock_dst);
                        }
                    }
                }
            }

            // Force software rendering if GL/Vulkan fails.
            if std::env::var("GSK_RENDERER").is_err() {
                std::env::set_var("GSK_RENDERER", "cairo");
            }
            if std::env::var("LIBGL_ALWAYS_SOFTWARE").is_err() {
                std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
            }
        }
    }

    fn build_css(&self) -> String {
        let (bg, fg) = match self.color_scheme {
            ColorScheme::Dark => ("#1d1d1d", "#ececec"),
            ColorScheme::Light => ("#ececec", "#1d1d1d"),
        };

        let mut css = format!(
            "window {{
                background-color: {};
                color: {};
            }}
            text, label {{
                color: {};
                font-family: 'SF Pro Display';
            }}
            button {{
                font-family: 'SF Pro Display';
            }}",
            bg, fg, fg,
        );

        if let Some((milkiness, alpha, _sigma)) = self.glass {
            let glass_alpha = alpha;
            let milk = (milkiness * 255.0) as u8;
            css.push_str(&format!(
                "\n.glass-panel {{
                    background: rgba({milk}, {milk}, {milk}, {glass_alpha:.2});
                    border-radius: 12px;
                    border: 1px solid rgba(255, 255, 255, 0.18);
                }}"
            ));
        }

        css
    }

    /// Run the application (blocking event loop).
    pub fn run(&mut self) {
        // Fix WSLg rendering: force Wayland socket link + software renderer.
        Self::fix_wslg_environment();

        let app = Application::builder()
            .application_id("org.tontoo.uikit")
            .build();

        let title = self.title.clone();
        let width = self.width;
        let height = self.height;
        let css = self.build_css();
        let tick = self.tick.take();
        let initial_view = if let Some(ref delegate) = self.delegate {
            Some(delegate.view())
        } else {
            self.root.take()
        };
        let initial_view = std::cell::RefCell::new(initial_view);

        // Store delegate in thread-local state for dispatch.
        if let Some(delegate) = self.delegate.take() {
            APP_STATE.with(|state| {
                *state.borrow_mut() = Some(AppState {
                    delegate,
                    window: None,
                });
            });
        }

        // Drive the per-frame tick (defaults to 60 FPS while the app runs).
        if let Some(mut tick) = tick {
            glib::timeout_add_local(Duration::from_millis(1000 / 60), move || {
                tick(1.0 / 60.0);
                glib::ControlFlow::Continue
            });
        }

        app.connect_activate(move |app| {
            // Load CSS.
            let css_provider = gtk::CssProvider::new();
            css_provider.load_from_string(&css);
            gtk::style_context_add_provider_for_display(
                &gtk::gdk::Display::default().expect("Could not get default display"),
                &css_provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION as u32,
            );

            // Build widget tree FIRST, then window.
            let gtk_widget = if let Some(view) = initial_view.borrow_mut().take() {
                Some(wrap_with_window_bar(view))
            } else {
                None
            };

            // Build window with child in builder.
            let mut builder = ApplicationWindow::builder()
                .application(app)
                .title(&title)
                .default_width(width)
                .default_height(height)
                .decorated(false);

            if let Some(ref child) = gtk_widget {
                builder = builder.child(child);
            }

            let window = builder.build();

            // Ensure the default cursor is visible (fixes invisible cursor on
            // WSLg / software rendering where the theme cursor may not show).
            window.set_cursor_from_name(Some("default"));

            // Store window in app state for dispatch.
            APP_STATE.with(|state| {
                let mut state = state.borrow_mut();
                if let Some(ref mut app) = *state {
                    app.window = Some(window.clone());
                }
            });

            window.present();
        });

        app.run();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_new() {
        let app = App::new("Test", 800, 600);
        assert_eq!(app.title(), "Test");
        assert_eq!(app.width(), 800);
        assert_eq!(app.height(), 600);
    }

    #[test]
    fn app_glass() {
        let mut app = App::new("Test", 800, 600);
        assert!(app.glass().is_none());
        app.set_glass(0.5, 0.7, 20.0);
        assert_eq!(app.glass(), Some((0.5, 0.7, 20.0)));
    }

    #[test]
    fn app_color_scheme() {
        let mut app = App::new("Test", 800, 600);
        assert_eq!(app.color_scheme(), ColorScheme::Dark);
        app.set_color_scheme(ColorScheme::Light);
        assert_eq!(app.color_scheme(), ColorScheme::Light);
    }
}
