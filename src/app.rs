//! Application runtime for TontooUIKit.
//!
//! Uses `gtk::Application` for the event loop and window management.
//! Delegate-driven state updates with automatic view rebuilding.

use crate::widget::Widget;
use glib::object::Cast;
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

impl ColorScheme {
    /// Detect the system color scheme from GTK4 settings.
    ///
    /// Checks the current theme name for "dark" suffix (e.g. "Adwaita-dark")
    /// and the `gtk-application-prefer-dark-theme` setting.
    pub fn detect_system() -> Self {
        if let Some(settings) = gtk::Settings::default() {
            // Check if dark theme is explicitly preferred
            if settings.property::<bool>("gtk-application-prefer-dark-theme") {
                return Self::Dark;
            }

            // Check theme name for dark variant
            let theme = settings.property::<String>("gtk-theme-name");
            let lower = theme.to_lowercase();
            if lower.ends_with("-dark") || lower.contains("dark") {
                return Self::Dark;
            }
        }
        Self::Light
    }
}

/// Application delegate trait — implement this for your app.
pub trait AppDelegate {
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
    show_window_bar: bool,
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
                        if window.is_fullscreen() {
                            window.unfullscreen();
                        } else {
                            window.fullscreen();
                        }
                    }
                    return;
                }
                _ => {}
            }

            // Delegate custom actions
            app.delegate.handle_custom(action_name);
            let view = app.delegate.view();
            let (content, _) = build_window_content(view, app.show_window_bar);
            let gtk_widget = wrap_window_with_resize_edges(content);
            gtk_widget.set_hexpand(true);
            gtk_widget.set_vexpand(true);
            gtk_widget.set_visible(true);
            if let Some(ref window) = app.window {
                // Drop old child first to avoid widget tree accumulation.
                window.set_child(None::<&gtk::Widget>);
                window.set_child(Some(&gtk_widget));
                window.queue_draw();
            }
        }
    });
}

/// Build the window content column: an optional traffic-light bar on top
/// (fixed) and the view below. Returns the column widget and the view's
/// natural size (used to size the window before it is shown).
fn build_window_content(view: Box<dyn Widget>, show_bar: bool) -> (gtk::Widget, (i32, i32)) {
    let view_widget = view.to_gtk();
    view_widget.set_hexpand(true);
    view_widget.set_vexpand(true);
    let (_, nat_w, _, _) = view_widget.measure(gtk::Orientation::Horizontal, -1);
    let (_, nat_h, _, _) = view_widget.measure(gtk::Orientation::Vertical, -1);

    // Split layout: an HStack root with exactly two children is treated as a
    // fixed sidebar (left) plus scrollable content (right). Only the content
    // scrolls; the sidebar stays fixed and fills the full window height.
    let root_layout = if let Some(b) = view_widget.downcast_ref::<gtk::Box>() {
        if b.orientation() == gtk::Orientation::Horizontal {
            let mut children = Vec::new();
            let mut c = b.first_child();
            while let Some(child) = c {
                children.push(child.clone());
                c = child.next_sibling();
            }
            if children.len() == 2 {
                let side = children[0].clone();
                let content = children[1].clone();
                b.remove(&side);
                b.remove(&content);

                let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                row.set_hexpand(true);
                row.set_vexpand(true);

                side.set_hexpand(false);
                side.set_vexpand(true);
                side.set_valign(gtk::Align::Fill);
                row.append(&side);

                let scrolled = wrap_scrollable(content);
                scrolled.set_hexpand(true);
                scrolled.set_vexpand(true);
                row.append(&scrolled);

                row.upcast()
            } else {
                wrap_scrollable(view_widget)
            }
        } else {
            wrap_scrollable(view_widget)
        }
    } else {
        wrap_scrollable(view_widget)
    };

    let column = gtk::Box::new(gtk::Orientation::Vertical, 0);
    column.set_hexpand(true);
    column.set_vexpand(true);
    if show_bar {
        let bar = crate::widgets::TrafficLights::new().to_gtk();
        bar.set_hexpand(true);
        column.append(&bar);
    }
    column.append(&root_layout);
    (column.upcast(), (nat_w, nat_h))
}

/// Wrap content in a scroll container so oversized content gets scrollbars
/// automatically instead of forcing the window to grow.
fn wrap_scrollable(content: gtk::Widget) -> gtk::Widget {
    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scrolled.set_child(Some(&content));
    scrolled.set_hexpand(true);
    scrolled.set_vexpand(true);
    scrolled.upcast()
}

/// Thickness of the invisible edge-resize strips in pixels.
const RESIZE_EDGE_PX: i32 = 6;
/// Size of the corner-resize squares in pixels.
const RESIZE_CORNER_PX: i32 = 14;

/// Window edges and corners that can be grabbed to resize an undecorated
/// window (the compositor provides no resize borders for them).
#[derive(Clone, Copy, PartialEq, Eq)]
enum ResizeEdge {
    North,
    South,
    West,
    East,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

impl ResizeEdge {
    fn cursor(self) -> &'static str {
        match self {
            Self::North => "n-resize",
            Self::South => "s-resize",
            Self::West => "w-resize",
            Self::East => "e-resize",
            Self::NorthWest => "nw-resize",
            Self::NorthEast => "ne-resize",
            Self::SouthWest => "sw-resize",
            Self::SouthEast => "se-resize",
        }
    }

    fn gdk_edge(self) -> gtk::gdk::SurfaceEdge {
        match self {
            Self::North => gtk::gdk::SurfaceEdge::North,
            Self::South => gtk::gdk::SurfaceEdge::South,
            Self::West => gtk::gdk::SurfaceEdge::West,
            Self::East => gtk::gdk::SurfaceEdge::East,
            Self::NorthWest => gtk::gdk::SurfaceEdge::NorthWest,
            Self::NorthEast => gtk::gdk::SurfaceEdge::NorthEast,
            Self::SouthWest => gtk::gdk::SurfaceEdge::SouthWest,
            Self::SouthEast => gtk::gdk::SurfaceEdge::SouthEast,
        }
    }

    fn apply_layout(self, area: &gtk::DrawingArea) {
        match self {
            Self::North | Self::South => {
                area.set_halign(gtk::Align::Fill);
                area.set_hexpand(true);
                area.set_height_request(RESIZE_EDGE_PX);
                if self == Self::North {
                    area.set_valign(gtk::Align::Start);
                } else {
                    area.set_valign(gtk::Align::End);
                }
            }
            Self::West | Self::East => {
                area.set_valign(gtk::Align::Fill);
                area.set_vexpand(true);
                area.set_width_request(RESIZE_EDGE_PX);
                if self == Self::West {
                    area.set_halign(gtk::Align::Start);
                } else {
                    area.set_halign(gtk::Align::End);
                }
            }
            _ => {
                area.set_size_request(RESIZE_CORNER_PX, RESIZE_CORNER_PX);
                area.set_halign(if matches!(self, Self::NorthWest | Self::SouthWest) {
                    gtk::Align::Start
                } else {
                    gtk::Align::End
                });
                area.set_valign(if matches!(self, Self::NorthWest | Self::NorthEast) {
                    gtk::Align::Start
                } else {
                    gtk::Align::End
                });
            }
        }
    }
}

/// Wrap the window content in an overlay with invisible resize handles on all
/// four edges and corners, so undecorated windows stay fully resizable.
fn wrap_window_with_resize_edges(content: gtk::Widget) -> gtk::Widget {
    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(&content));
    overlay.set_hexpand(true);
    overlay.set_vexpand(true);

    for edge in [
        ResizeEdge::North,
        ResizeEdge::South,
        ResizeEdge::West,
        ResizeEdge::East,
    ] {
        overlay.add_overlay(&resize_handle(edge));
    }
    // Corners last so they sit on top of the edge strips.
    for corner in [
        ResizeEdge::NorthWest,
        ResizeEdge::NorthEast,
        ResizeEdge::SouthWest,
        ResizeEdge::SouthEast,
    ] {
        overlay.add_overlay(&resize_handle(corner));
    }

    overlay.upcast()
}

/// An invisible strip along one window edge. On press the interactive resize
/// is handed to the compositor via `gdk_toplevel_begin_resize` (smooth on
/// Wayland/X11). The same gesture drives a manual `set_default_size` fallback
/// that takes over whenever the compositor handoff has no effect. GestureDrag
/// keeps tracking even when the pointer leaves the strip.
fn resize_handle(edge: ResizeEdge) -> gtk::Widget {
    let area = gtk::DrawingArea::new();
    edge.apply_layout(&area);
    area.set_cursor_from_name(Some(edge.cursor()));
    area.set_can_focus(false);

    let start_size = std::rc::Rc::new(std::cell::Cell::new((0i32, 0i32)));

    let drag = gtk::GestureDrag::new();
    drag.set_button(1);

    // Drag begin: remember the window size and try the compositor-driven
    // resize first.
    {
        let ss = start_size.clone();
        let area_wk = area.downgrade();
        drag.connect_drag_begin(move |gesture, gx, gy| {
            let Some(a) = area_wk.upgrade() else { return };
            let Some(win) = a.root().and_then(|r| r.downcast::<gtk::Window>().ok()) else { return };
            ss.set((win.width(), win.height()));

            let event = gesture.current_event();
            let device = event
                .as_ref()
                .and_then(|e| e.device())
                .or_else(|| win.display().default_seat().and_then(|s| s.pointer()));
            let time = event.as_ref().map(|e| e.time()).unwrap_or(0);
            let Some(surface) = win.surface() else { return };
            let Some(toplevel) = surface.dynamic_cast_ref::<gtk::gdk::Toplevel>() else { return };

            // gdk_toplevel_begin_resize expects root (screen) coordinates on
            // X11 (_NET_WM_MOVERESIZE); passing widget-local ones makes it a
            // silent no-op, so translate them via the pointer position.
            let (rx, ry) = match device.as_ref() {
                Some(d) => surface
                    .device_position(d)
                    .map(|(x, y, _)| (x, y))
                    .unwrap_or((gx, gy)),
                None => (gx, gy),
            };
            match device.as_ref() {
                Some(d) => toplevel.begin_resize(edge.gdk_edge(), Some(d), 1, rx, ry, time),
                None => {
                    toplevel.begin_resize(edge.gdk_edge(), None::<&gtk::gdk::Device>, 1, rx, ry, time)
                }
            }
        });
    }

    // Drag update: manual fallback. If the compositor accepted the handoff it
    // owns the pointer grab, so no further events reach this gesture and this
    // stays dormant.
    {
        let ss = start_size.clone();
        let area_wk = area.downgrade();
        drag.connect_drag_update(move |_, dx, dy| {
            let Some(a) = area_wk.upgrade() else { return };
            let Some(win) = a.root().and_then(|r| r.downcast::<gtk::Window>().ok()) else { return };
            let (ow, oh) = ss.get();
            if ow == 0 && oh == 0 {
                return;
            }
            let dx = dx.round() as i32;
            let dy = dy.round() as i32;
            let mut nw = ow;
            let mut nh = oh;
            match edge {
                ResizeEdge::East | ResizeEdge::NorthEast | ResizeEdge::SouthEast => nw = ow + dx,
                ResizeEdge::West | ResizeEdge::NorthWest | ResizeEdge::SouthWest => nw = ow - dx,
                _ => {}
            }
            match edge {
                ResizeEdge::South | ResizeEdge::SouthEast | ResizeEdge::SouthWest => nh = oh + dy,
                ResizeEdge::North | ResizeEdge::NorthEast | ResizeEdge::NorthWest => nh = oh - dy,
                _ => {}
            }
            win.set_default_size(nw.max(320), nh.max(240));
        });
    }

    area.add_controller(drag);
    area.upcast()
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
    show_window_bar: bool,
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
            show_window_bar: true,
            tick: None,
        }
    }

    pub fn with_delegate(
        title: impl Into<String>,
        width: i32,
        height: i32,
        delegate: impl AppDelegate + 'static,
    ) -> Self {
        let mut app = Self::new(title, width, height);
        app.delegate = Some(Box::new(delegate));
        app
    }

    pub fn set_root(&mut self, widget: impl Widget + 'static) {
        self.root = Some(Box::new(widget));
    }

    /// Set the root view (new View-based API).
    pub fn set_root_view(&mut self, view: crate::view::View) {
        struct ViewWrapper(crate::view::View);
        impl Widget for ViewWrapper {
            fn id(&self) -> crate::widget::WidgetId { 0 }
            fn to_gtk(&self) -> gtk::Widget { self.0.to_gtk() }
        }
        self.root = Some(Box::new(ViewWrapper(view)));
    }

    /// Set the application delegate (Apple UIKit style).
    pub fn set_delegate(&mut self, delegate: impl AppDelegate + 'static) {
        self.delegate = Some(Box::new(delegate));
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

    /// Disable the automatic window bar (traffic lights + drag area).
    pub fn no_window_bar(&mut self) -> &mut Self {
        self.show_window_bar = false;
        self
    }

    /// Auto-detect and set the color scheme from system settings.
    pub fn auto_color_scheme(&mut self) {
        self.color_scheme = ColorScheme::detect_system();
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
    /// # use uikit::prelude::*;
    /// # use uikit::animation::*;
    /// # use uikit::style::Size;
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

    /// Fix WSLg rendering issues by linking Wayland sockets and preferring
    /// the OpenGL renderer (the cursor only renders via the Wayland path).
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

            // Prefer native Wayland. On WSLg the OpenGL renderer can freeze
            // input handling, so use cairo (software) which is reliable.
            if std::env::var("GSK_RENDERER").is_err() {
                std::env::set_var("GSK_RENDERER", "cairo");
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
            }}
            scrollbar {{
                background: transparent;
                border: none;
            }}
            scrollbar slider {{
                min-width: 6px;
                min-height: 40px;
                border-radius: 3px;
                background: rgba(255, 255, 255, 0.2);
                border: none;
                margin: 2px;
            }}
            scrollbar slider:hover {{
                background: rgba(255, 255, 255, 0.35);
            }}
            scrollbar slider:active {{
                background: rgba(255, 255, 255, 0.5);
            }}
            scrolledwindow {{
                background: transparent;
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
        let has_delegate = self.delegate.is_some();
        let initial_root = std::cell::RefCell::new(self.root.take());

        // Store delegate in thread-local state for dispatch.
        let delegate = self.delegate.take().unwrap_or_else(|| Box::new(NoDelegate));
        APP_STATE.with(|state| {
            *state.borrow_mut() = Some(AppState {
                delegate,
                window: None,
                show_window_bar: self.show_window_bar,
            });
        });

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

            // Build widget tree AFTER GTK is initialized.
            let initial_view = if has_delegate {
                APP_STATE.with(|state| {
                    let state = state.borrow();
                    state.as_ref().map(|s| s.delegate.view())
                })
            } else {
                initial_root.borrow_mut().take()
            };

            let mut content_nat: (i32, i32) = (0, 0);
            let gtk_widget: Option<gtk::Widget> = initial_view.map(|view| {
                let state = APP_STATE.with(|s| s.borrow().as_ref().map(|s| s.show_window_bar).unwrap_or(true));
                let (content, nat) = build_window_content(view, state);
                content_nat = nat;
                // Add invisible edge/corner resize handles around the content.
                wrap_window_with_resize_edges(content)
            });

            // The view is already wrapped in its own scroll container inside
            // build_window_content, so the window child is ready to use.
            let child: Option<gtk::Widget> = gtk_widget;

            // Build window with child in builder.
            let mut builder = ApplicationWindow::builder()
                .application(app)
                .title(&title)
                .default_width(width)
                .default_height(height)
                .decorated(false);

            if let Some(ref child) = child {
                builder = builder.child(child);
            }

            let window = builder.build();

            // Allow window resizing even when undecorated.
            window.set_resizable(true);

// Default the window to the content's natural size, capped at half the
            // monitor size in both directions. The window therefore never
            // grows to show everything when the content is huge — oversized
            // content scrolls instead. The app can still resize afterwards.
            let display = gtk::prelude::WidgetExt::display(&window);
            let monitor = display
                .monitors()
                .item(0)
                .and_then(|o| o.downcast::<gtk::gdk::Monitor>().ok())
                .or_else(|| {
                    window
                        .surface()
                        .and_then(|s| display.monitor_at_surface(&s))
                });
            if let Some(monitor) = monitor {
                // Divide by the monitor scale factor first: WSLg / HiDPI
                // reports the virtual resolution (e.g. 2560x1440) while the
                // visible area is half that, so the cap must use the scaled
                // (visible) size to actually be half the on-screen space.
                let g = monitor.geometry();
                let scale = monitor.scale_factor().max(1);
                let vis_w = g.width() / scale;
                let vis_h = g.height() / scale;
                let cap_w = (vis_w / 2).max(320);
                let cap_h = (vis_h / 2).max(240);
                let (nat_w, nat_h) = content_nat;
                let dw = if nat_w > 0 { nat_w.min(cap_w) } else { width.min(cap_w) };
                let dh = if nat_h > 0 { nat_h.min(cap_h) } else { height.min(cap_h) };
                window.set_default_size(dw.max(320), dh.max(240));
            }

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
