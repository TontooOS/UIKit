//! Application runtime for TontooUIKit.
//!
//! Uses `gtk::Application` for the event loop and window management.
//! Delegate-driven state updates with automatic view rebuilding.

use crate::widget::Widget;
use glib::object::Cast;
use gtk::prelude::*;
use gtk::{self, Application, ApplicationWindow};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

/// Scheme of the currently running App: 0 = none, 1 = dark, 2 = light.
static CURRENT_SCHEME: AtomicU8 = AtomicU8::new(0);

/// Toolkit identity reported to the system (see [`mark_toolkit`]).
///
/// CoreWindows reads this value from `/proc/<pid>/environ` to classify open
/// windows by toolkit (`UIKit`, `TontooUI`, ...). Every TontooOS UI toolkit
/// sets the same variable with its own id so classification stays uniform.
pub const TOOLKIT_ENV_VAR: &str = "TONTOO_TOOLKIT";

/// Toolkit id of this library.
pub const TOOLKIT_ID: &str = "UIKit";

/// Publish this process as a UIKit app for system services.
///
/// Sets `TONTOO_TOOLKIT=UIKit` in the process environment. Called
/// automatically by [`App::run`]; call it manually when driving GTK without
/// `App` (custom event loops) so CoreWindows still classifies the windows.
pub fn mark_toolkit() {
    // SAFETY: single-threaded at startup in practice (`App::run` calls this
    // before spawning threads); `set_var` is only unsafe in the 2024 edition.
    unsafe { std::env::set_var(TOOLKIT_ENV_VAR, TOOLKIT_ID) };
}

fn scheme_to_u8(scheme: ColorScheme) -> u8 {
    match scheme {
        ColorScheme::Dark => 1,
        ColorScheme::Light => 2,
    }
}

/// The color scheme of the running [`App`], if one has been set.
///
/// Elements rendered without an explicit override read this so they match
/// the app's look. Returns `None` outside of a running App.
pub fn current_color_scheme() -> Option<ColorScheme> {
    match CURRENT_SCHEME.load(Ordering::Relaxed) {
        1 => Some(ColorScheme::Dark),
        2 => Some(ColorScheme::Light),
        _ => None,
    }
}

/// Whether the window decoration bar is currently shown.
///
/// Views that draw their own traffic lights (e.g. TontooUI `Sidebar`) read
/// this at render time and drop their own lights when the bar is on.
/// Returns `false` outside of a running App (standalone previews keep
/// drawing their own lights, as before).
pub fn is_window_bar_visible() -> bool {
    APP_STATE.with(|state| {
        state
            .borrow()
            .as_ref()
            .map(|app| app.show_window_bar)
            .unwrap_or(false)
    })
}

/// Effective decoration-bar visibility: a view tree that draws its own
/// traffic lights hides the system bar automatically, unless explicitly
/// forced on. Pure helper so the rule stays unit-testable without GTK.
pub(crate) fn effective_window_bar(
    user_show_bar: bool,
    force_window_bar: bool,
    tree_hides_bar: bool,
) -> bool {
    force_window_bar || (user_show_bar && !tree_hides_bar)
}

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
    /// Detect the system color scheme from GTK4 settings + GNOME portal.
    ///
    /// Checks `gtk-application-prefer-dark-theme`, theme name, GNOME 42+
    /// `org.gnome.desktop.interface color-scheme` (via `gsettings`), and
    /// fallback `settings.ini`. Safe to call before `gtk::init`.
    pub fn detect_system() -> Self {
        // If GTK not yet initialized, try to init silently so Settings is available
        if !gtk::is_initialized() {
            let _ = gtk::init();
        }
        if gtk::is_initialized() {
            if let Some(settings) = gtk::Settings::default() {
                if settings.property::<bool>("gtk-application-prefer-dark-theme") {
                    return Self::Dark;
                }
                let theme = settings.property::<String>("gtk-theme-name");
                let lower = theme.to_lowercase();
                if lower.ends_with("-dark") || lower.contains("dark") {
                    return Self::Dark;
                }
            }
            // GNOME 42+: org.gnome.desktop.interface color-scheme = 'prefer-dark'
            #[cfg(target_os = "linux")]
            {
                if let Ok(out) = std::process::Command::new("gsettings")
                    .args(["get", "org.gnome.desktop.interface", "color-scheme"])
                    .output()
                {
                    let s = String::from_utf8_lossy(&out.stdout).to_lowercase();
                    if s.contains("prefer-dark") {
                        return Self::Dark;
                    }
                    if s.contains("prefer-light") {
                        return Self::Light;
                    }
                }
            }
            return Self::Light;
        }
        // Fallback when GTK unavailable (pre-init / headless) — TontooOS default is Dark (#1E1E1E)
        if let Ok(home) = std::env::var("HOME") {
            for path in [
                format!("{}/.config/gtk-4.0/settings.ini", home),
                "/etc/gtk-4.0/settings.ini".to_string(),
            ] {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let lower = content.to_lowercase();
                    if lower.contains("prefer-dark-theme=true") || (lower.contains("gtk-theme-name") && lower.contains("dark")) {
                        return Self::Dark;
                    }
                }
            }
        }
        Self::Dark
    }

    /// System background for current scheme (TontooOS standard: light #F5F5F7, dark #1E1E1E).
    pub fn bg_hex(self) -> &'static str {
        match self {
            Self::Dark => "#1E1E1E",
            Self::Light => "#F5F5F7",
        }
    }
    /// System foreground for current scheme.
    pub fn fg_hex(self) -> &'static str {
        match self {
            Self::Dark => "#F5F5F7",
            Self::Light => "#1E1E1E",
        }
    }
}

/// Window chrome type.
///
/// - `Standard` (default): current UIKit window — slim traffic-light bar,
///   opaque background, no glass. Existing code is unaffected.
/// - `Mac`: macOS-style window — taller decoration bar with centered title,
///   built for the glass look (glass itself stays opt-in via
///   `set_glass_strength` / `set_window_transparency` / `set_window_blur`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowType {
    #[default]
    Standard,
    Mac,
}

impl WindowType {
    /// Title bar height in px for this window type.
    pub const fn title_bar_height(self) -> i32 {
        match self {
            Self::Standard => 31,
            Self::Mac => 44,
        }
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
    force_window_bar: bool,
    resizable: bool,
    scroll_content: bool,
    title: String,
    window_type: WindowType,
    /// Current inline title bar (always visible, both modes). Backend only.
    titlebar: Option<gtk::Widget>,
    /// Custom title-bar widget (fills the bar after the reserved traffic
    /// lights). Rendered on every rebuild.
    titlebar_widget: Option<Rc<dyn Widget>>,
    /// Show the title-bar title text. Only renders without a custom
    /// widget. Default true.
    titlebar_show_title: bool,
}

thread_local! {
    static APP_STATE: RefCell<Option<AppState>> = RefCell::new(None);
}

/// Called by buttons when a custom action fires.
pub fn dispatch_custom(action_name: &str) {
    // Snapshot the window and action under a short shared borrow, then act
    // with no borrow held: window methods (close/minimize/fullscreen) run
    // GTK destroy/notify handlers synchronously, and those re-enter
    // APP_STATE with borrow_mut() (e.g. build_window_content). Holding a
    // borrow across the window call panics with "RefCell already borrowed".
    let snapshot: Option<(Option<gtk::ApplicationWindow>, String)> = APP_STATE.with(|state| {
        state
            .borrow()
            .as_ref()
            .map(|app| (app.window.clone(), action_name.to_string()))
    });
    let Some((window, action)) = snapshot else {
        return;
    };
    match action.as_str() {
        "__close" => {
            if let Some(window) = window {
                window.close();
            }
        }
        "__minimize" => {
            if let Some(window) = window {
                window.minimize();
            }
        }
        "__maximize" => {
            if let Some(window) = window {
                if window.is_fullscreen() {
                    window.unfullscreen();
                } else {
                    window.fullscreen();
                }
            }
        }
        custom => {
            // Delegate custom actions (fresh borrow, no window call inside).
            APP_STATE.with(|state| {
                if let Some(ref mut app) = *state.borrow_mut() {
                    app.delegate.handle_custom(custom);
                }
            });
        }
    }
    // Rebuild content for the current mode (windowed or fullscreen).
    let fullscreen = APP_STATE.with(|state| {
        state
            .borrow()
            .as_ref()
            .and_then(|app| app.window.as_ref())
            .map(|window| window.is_fullscreen())
            .unwrap_or(false)
    });
    apply_window_chrome(fullscreen);
}

/// Backend action for the macOS-style chrome keys. F11 toggles fullscreen
/// in both modes, ESC leaves fullscreen, everything else is ignored.
/// No public API change: only `App::run` wires this to the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FullscreenKeyAction {
    Toggle,
    Exit,
    Ignored,
}

fn fullscreen_key_action(key: gtk::gdk::Key, fullscreen: bool) -> FullscreenKeyAction {
    if key == gtk::gdk::Key::F11 {
        FullscreenKeyAction::Toggle
    } else if key == gtk::gdk::Key::Escape && fullscreen {
        FullscreenKeyAction::Exit
    } else {
        FullscreenKeyAction::Ignored
    }
}

/// Reveal strip height (px) for the fullscreen traffic lights.
/// macOS shows the lights while the pointer touches the top edge.
const FULLSCREEN_REVEAL_PX: f64 = 32.0;

fn fullscreen_reveal_lights(pointer_y: f64) -> bool {
    pointer_y < FULLSCREEN_REVEAL_PX
}

/// Title-bar options for [`build_window_content`] (backend only).
struct TitleBarConfig {
    custom: Option<Rc<dyn Widget>>,
    show_title: bool,
    minimize_enabled: bool,
}

/// Find the traffic-lights container inside a title bar (tagged
/// `uikit-lights` in `TrafficLights::to_gtk`). Backend only.
fn find_lights_widget(root: &gtk::Widget) -> Option<gtk::Widget> {
    if root.widget_name() == "uikit-lights" {
        return Some(root.clone());
    }
    let mut child = root.first_child();
    while let Some(current) = child {
        if let Some(found) = find_lights_widget(&current) {
            return Some(found);
        }
        child = current.next_sibling();
    }
    None
}

/// Show or hide the traffic lights inside a visible title bar.
/// Hidden lights keep their layout space (opacity instead of visibility),
/// so button positions never shift; they also ignore input while hidden.
fn set_bar_lights_visible(bar: &gtk::Widget, visible: bool) {
    if let Some(lights) = find_lights_widget(bar) {
        lights.set_opacity(if visible { 1.0 } else { 0.0 });
        lights.set_sensitive(visible);
    }
}

/// (Re)build the window child for windowed or fullscreen mode (backend
/// only, no public API change).
///
/// The decoration bar stays visible in both modes with identical button
/// positions. Fullscreen follows macOS: resize edges are dropped, the
/// traffic lights hide (keeping layout space) until the pointer touches
/// the top edge, and the minimize button is disabled (gray, no-op).
fn apply_window_chrome(fullscreen: bool) {
    APP_STATE.with(|state| {
        let mut state = state.borrow_mut();
        let app = match state.as_mut() {
            Some(app) => app,
            None => return,
        };
        let window = match app.window.clone() {
            Some(window) => window,
            None => return,
        };
        let view = app.delegate.view();
        let title = app.title.clone();
        let bar_config = TitleBarConfig {
            custom: app.titlebar_widget.clone(),
            show_title: app.titlebar_show_title,
            minimize_enabled: !fullscreen,
        };
        let (mut content, _, bar) = build_window_content(
            view,
            app.show_window_bar,
            app.scroll_content,
            &title,
            app.window_type,
            bar_config,
        );
        if app.resizable && !fullscreen {
            content = wrap_window_with_resize_edges(content);
        }
        content.set_hexpand(true);
        content.set_vexpand(true);
        content.set_visible(true);
        if let Some(ref bar) = bar {
            bar.set_hexpand(true);
            bar.set_visible(true);
            set_bar_lights_visible(bar, !fullscreen);
        }
        app.titlebar = bar;
        window.set_child(Some(&content));
        window.queue_draw();
    });
}

/// Build the window content column: an optional traffic-light bar on top
/// (fixed) and the view below. Returns the column widget, the view's
/// natural size (used to size the window before it is shown) and the bar
/// widget. `bar_config` fills the bar after the reserved traffic lights
/// and controls title/minimize rendering. When `scroll`
/// is false the view is used as-is without any scroll container.
fn build_window_content(view: Box<dyn Widget>, show_bar: bool, scroll: bool, title: &str, window_type: WindowType, bar_config: TitleBarConfig) -> (gtk::Widget, (i32, i32), Option<gtk::Widget>) {
    // A view tree that draws its own traffic lights (e.g. TontooUI `Sidebar`)
    // hides the system decoration bar automatically, unless explicitly forced
    // on. The effective value is written back so views rendered below (which
    // read `is_window_bar_visible`) stay consistent: exactly one set of
    // traffic lights. Idempotent across rebuilds.
    let show_bar = APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        match state.as_mut() {
            Some(app) => {
                let effective = effective_window_bar(
                    app.show_window_bar,
                    app.force_window_bar,
                    view.hides_window_bar_recursive(),
                );
                app.show_window_bar = effective;
                effective
            }
            None => show_bar,
        }
    });
    let view_widget = view.to_gtk();
    view_widget.set_hexpand(true);
    view_widget.set_vexpand(true);
    let (_, nat_w, _, _) = view_widget.measure(gtk::Orientation::Horizontal, -1);
    let (_, nat_h, _, _) = view_widget.measure(gtk::Orientation::Vertical, -1);

    // Split layout: an HStack root with exactly two children is treated as a
    // fixed sidebar (left) plus scrollable content (right). Only the content
    // scrolls; the sidebar stays fixed and fills the full window height.
    // With scrolling disabled the view is embedded directly (fixed windows).
    let root_layout = if !scroll {
        view_widget
    } else if let Some(b) = view_widget.downcast_ref::<gtk::Box>() {
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
    let mut bar_widget: Option<gtk::Widget> = None;
    if show_bar {
        let mut lights = crate::widgets::TrafficLights::new()
            .with_title(title)
            .bar_height(window_type.title_bar_height() as f32)
            .show_title(bar_config.show_title)
            .minimize_enabled(bar_config.minimize_enabled);
        if let Some(custom) = bar_config.custom {
            lights = lights.with_custom_shared(custom);
        }
        let bar = lights.to_gtk();
        bar.set_hexpand(true);
        // Solid title bar: opaque scheme background via `.uikit-titlebar`
        // (beats the widget's own transparent `windowhandle` rule).
        bar.add_css_class("uikit-titlebar");
        column.append(&bar);
        bar_widget = Some(bar);
    }
    column.append(&root_layout);
    // Include the title bar height so the window fits bar + content.
    let nat = if show_bar {
        (nat_w, nat_h + window_type.title_bar_height())
    } else {
        (nat_w, nat_h)
    };
    (column.upcast(), nat, bar_widget)
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

/// Resolve the window default size: the requested size is the base, grown
/// to fit the content natural size when larger, capped at half the monitor
/// so huge content scrolls instead of opening a giant window.
fn resolve_default_size(
    nat_w: i32,
    nat_h: i32,
    req_w: i32,
    req_h: i32,
    cap_w: i32,
    cap_h: i32,
) -> (i32, i32) {
    let dw = nat_w.max(req_w).min(cap_w).max(320);
    let dh = nat_h.max(req_h).min(cap_h).max(240);
    (dw, dh)
}

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

    let debug = std::env::var("UIKIT_RESIZE_DEBUG").is_ok();
    let mut handles: Vec<(ResizeEdge, gtk::Widget)> = Vec::new();

    for edge in [
        ResizeEdge::North,
        ResizeEdge::South,
        ResizeEdge::West,
        ResizeEdge::East,
    ] {
        let h = resize_handle(edge);
        handles.push((edge, h.clone()));
        overlay.add_overlay(&h);
    }
    // Corners last so they sit on top of the edge strips.
    for corner in [
        ResizeEdge::NorthWest,
        ResizeEdge::NorthEast,
        ResizeEdge::SouthWest,
        ResizeEdge::SouthEast,
    ] {
        let h = resize_handle(corner);
        handles.push((corner, h.clone()));
        overlay.add_overlay(&h);
    }

    if debug {
        for (edge, h) in &handles {
            apply_debug_visuals(edge, h);
        }
        let overlay_wk = overlay.downgrade();
        glib::timeout_add_local(std::time::Duration::from_millis(1500), move || {
            if let Some(ov) = overlay_wk.upgrade() {
                for (edge, h) in &handles {
                    let pos = h.translate_coordinates(&ov, 0.0, 0.0);
                    eprintln!(
                        "DBG alloc {:?} pos={:?} size={}x{}",
                        edge_to_str(*edge),
                        pos,
                        h.allocated_width(),
                        h.allocated_height()
                    );
                }
            }
            glib::ControlFlow::Break
        });
    }

    overlay.upcast()
}

fn edge_to_str(e: ResizeEdge) -> &'static str {
    match e {
        ResizeEdge::North => "N",
        ResizeEdge::South => "S",
        ResizeEdge::West => "W",
        ResizeEdge::East => "E",
        ResizeEdge::NorthWest => "NW",
        ResizeEdge::NorthEast => "NE",
        ResizeEdge::SouthWest => "SW",
        ResizeEdge::SouthEast => "SE",
    }
}

fn apply_debug_visuals(edge: &ResizeEdge, area: &gtk::Widget) {
    use crate::widget::apply_css;
    let color = match edge {
        ResizeEdge::North => "#ff333388",
        ResizeEdge::South => "#33ff3388",
        ResizeEdge::West => "#3333ff88",
        ResizeEdge::East => "#ffff3388",
        ResizeEdge::NorthWest | ResizeEdge::NorthEast => "#ff00ff88",
        ResizeEdge::SouthWest | ResizeEdge::SouthEast => "#00ffffff",
    };
    let css = format!("window {{ background-image: none; }} drawingarea {{ background-color: {color}; }}");
    apply_css(area, &css);
}

/// Minimal X11 helpers used to move the window during west/north resize
/// drags (GDK4 exposes no public move API). The connection is cached for the
/// process lifetime.
#[cfg(target_os = "linux")]
mod x11_pos {
    use std::sync::OnceLock;
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{ConnectionExt, ConfigureWindowAux};

    type Conn = x11rb::rust_connection::RustConnection;

    fn connection() -> Option<&'static (Conn, usize)> {
        static CONN: OnceLock<Option<(Conn, usize)>> = OnceLock::new();
        CONN.get_or_init(|| x11rb::connect(None).ok()).as_ref()
    }

    /// X window id of a GDK surface, via libgtk-4's exported symbol (no
    /// separate gdk4-x11 crate dependency needed).
    pub fn xid_of(surface: &gtk::gdk::Surface) -> Option<u32> {
        extern "C" {
            fn gdk_x11_surface_get_xid(surface: *const std::ffi::c_void) -> u32;
        }
        use glib::translate::{ToGlibPtr, Stash};
        let ptr: Stash<'_, *const gtk::gdk::ffi::GdkSurface, _> = surface.to_glib_none();
        let xid = unsafe { gdk_x11_surface_get_xid(ptr.0.cast()) };
        Some(xid)
    }

    /// Current top-left origin of the X window in root coordinates.
    pub fn root_origin(xid: u32) -> Option<(i32, i32)> {
        let (conn, screen) = connection()?;
        let root = conn.setup().roots[*screen].root;
        let reply = conn
            .translate_coordinates(xid, root, 0, 0)
            .ok()?
            .reply()
            .ok()?;
        Some((reply.dst_x as i32, reply.dst_y as i32))
    }

    /// Move the window (position only; GTK keeps owning the size).
    pub fn move_window(xid: u32, x: i32, y: i32) -> bool {
        let Some((conn, _)) = connection() else { return false };
        conn.configure_window(xid, &ConfigureWindowAux::new().x(x).y(y))
            .is_ok()
    }
}

#[cfg(not(target_os = "linux"))]
mod x11_pos {
    pub fn xid_of(_surface: &gtk::gdk::Surface) -> Option<u32> {
        None
    }
    pub fn root_origin(_xid: u32) -> Option<(i32, i32)> {
        None
    }
    pub fn move_window(_xid: u32, _x: i32, _y: i32) -> bool {
        false
    }
}

/// Minimum window size applied to manual resizes.
const RESIZE_MIN_W: i32 = 320;
/// Minimum window height applied to manual resizes.
const RESIZE_MIN_H: i32 = 240;

/// Pure geometry for one manual resize step (X11 path): the gesture deltas
/// (`dx`, `dy`, logical pixels from drag start) plus the size at drag start
/// (`ow`, `oh`) yield the new default size and whether the window origin must
/// move to keep the opposite edge anchored. Unit-testable without GTK.
fn manual_resize_geometry(
    edge: ResizeEdge,
    ow: i32,
    oh: i32,
    dx: i32,
    dy: i32,
) -> (i32, i32, bool, bool) {
    let mut nw = ow;
    let mut nh = oh;
    let mut move_x = false;
    let mut move_y = false;
    match edge {
        ResizeEdge::East | ResizeEdge::NorthEast | ResizeEdge::SouthEast => nw = ow + dx,
        ResizeEdge::West | ResizeEdge::NorthWest | ResizeEdge::SouthWest => {
            nw = ow - dx;
            move_x = true;
        }
        _ => {}
    }
    match edge {
        ResizeEdge::South | ResizeEdge::SouthEast | ResizeEdge::SouthWest => nh = oh + dy,
        ResizeEdge::North | ResizeEdge::NorthEast | ResizeEdge::NorthWest => {
            nh = oh - dy;
            move_y = true;
        }
        _ => {}
    }
    (nw.max(RESIZE_MIN_W), nh.max(RESIZE_MIN_H), move_x, move_y)
}

/// Repositioning for west/north drags: move by the size change that was
/// actually applied (after min-size clamping) so the opposite edge stays
/// anchored instead of drifting when the minimum size is hit. `scale`
/// converts logical pixels to physical ones for the X11 root window.
fn anchored_origin(
    origin: (i32, i32),
    start: (i32, i32),
    current: (i32, i32),
    move_x: bool,
    move_y: bool,
    scale: f64,
) -> (i32, i32) {
    let (ox, oy) = origin;
    let nx = if move_x {
        ox + ((start.0 - current.0) as f64 * scale).round() as i32
    } else {
        ox
    };
    let ny = if move_y {
        oy + ((start.1 - current.1) as f64 * scale).round() as i32
    } else {
        oy
    };
    (nx, ny)
}

/// An invisible strip along one window edge. Exactly one backend owns the
/// resize, never both at once: native Wayland delegates the whole interactive
/// resize to the compositor (`gdk_toplevel_begin_resize`), which anchors every
/// edge correctly. Everywhere else (X11/XWayland/WSLg) the gesture resizes
/// manually with `set_default_size` and — for west/north edges — moves the
/// window through X11 so the grabbed edge follows the pointer instead of the
/// opposite side growing. Running both mechanisms at once makes the window
/// snap back on button release (the compositor commits its own stale geometry
/// over the manual size), so the manual fallback stays off while the
/// compositor owns the drag. GestureDrag keeps tracking even when the pointer
/// leaves the strip.
fn resize_handle(edge: ResizeEdge) -> gtk::Widget {
    let area = gtk::DrawingArea::new();
    edge.apply_layout(&area);
    area.set_cursor_from_name(Some(edge.cursor()));
    area.set_can_focus(false);

    let start_size = std::rc::Rc::new(std::cell::Cell::new((0i32, 0i32)));
    let origin = std::rc::Rc::new(std::cell::Cell::new(None::<(i32, i32)>));
    let xid = std::rc::Rc::new(std::cell::Cell::new(None::<u32>));
    /// True while the compositor owns the resize (native Wayland): the manual
    /// `set_default_size` fallback must stay off so both mechanisms never
    /// fight over the window size.
    let delegated = std::rc::Rc::new(std::cell::Cell::new(false));

    let drag = gtk::GestureDrag::new();
    drag.set_button(1);

    // Drag begin: record size. Native Wayland delegates the whole resize to
    // the compositor; X11 resizes purely manually (no `begin_resize` probe,
    // whose window-manager session would otherwise revert the manual size on
    // release).
    {
        let ss = start_size.clone();
        let origin = origin.clone();
        let xid_cell = xid.clone();
        let delegated = delegated.clone();
        let area_wk = area.downgrade();
        drag.connect_drag_begin(move |gesture, gx, gy| {
            let Some(a) = area_wk.upgrade() else { return };
            let Some(win) = a.root().and_then(|r| r.downcast::<gtk::Window>().ok()) else { return };
            ss.set((win.width(), win.height()));
            origin.set(None);
            xid_cell.set(None);
            delegated.set(false);

            let is_wayland = gtk::prelude::RootExt::display(&win)
                .type_()
                .name()
                .starts_with("GdkWayland");

            // gdk_toplevel_begin_resize expects surface-local press
            // coordinates plus the press device so GDK can pass the matching
            // implicit-grab serial to the compositor.
            let Some(surface) = win.surface() else { return };

            if is_wayland {
                let Some(toplevel) = surface.dynamic_cast_ref::<gtk::gdk::Toplevel>() else {
                    return;
                };
                let event = gesture.current_event();
                let device = event
                    .as_ref()
                    .and_then(|e| e.device())
                    .or_else(|| {
                        gtk::prelude::RootExt::display(&win)
                            .default_seat()
                            .and_then(|s| s.pointer())
                    });
                let time = event.as_ref().map(|e| e.time()).unwrap_or(0);
                // The gesture reports handle-local coordinates (a few px
                // inside a 6 px strip), but `begin_resize` needs surface-local
                // ones. Translate into the window so the compositor anchors
                // the correct edge (undecorated: window == surface).
                let (sx, sy) = a
                    .translate_coordinates(&win, gx, gy)
                    .unwrap_or((gx, gy));
                match device.as_ref() {
                    Some(d) => toplevel.begin_resize(edge.gdk_edge(), Some(d), 1, sx, sy, time),
                    None => toplevel.begin_resize(
                        edge.gdk_edge(),
                        None::<&gtk::gdk::Device>,
                        1,
                        sx,
                        sy,
                        time,
                    ),
                }
                // Compositor owns the pointer grab now; no manual fallback.
                delegated.set(true);
                return;
            }

            // X11 path: capture the X window id and its current root origin
            // once, so west/north drags can keep the grabbed edge anchored.
            // No compositor handoff here: the gesture below owns the size.
            if let Some(x) = x11_pos::xid_of(&surface) {
                xid_cell.set(Some(x));
                origin.set(x11_pos::root_origin(x));
            }
        });
    }

    // Drag update: manual resize (X11 path only; skipped while the
    // compositor owns the drag). West/north drags additionally reposition the
    // window so the grabbed edge tracks the pointer.
    {
        let ss = start_size.clone();
        let origin = origin.clone();
        let xid_cell = xid.clone();
        let delegated = delegated.clone();
        let area_wk = area.downgrade();
        drag.connect_drag_update(move |_, dx, dy| {
            if delegated.get() {
                return;
            }
            let Some(a) = area_wk.upgrade() else { return };
            let Some(win) = a.root().and_then(|r| r.downcast::<gtk::Window>().ok()) else { return };

            let (ow, oh) = ss.get();
            if ow == 0 && oh == 0 {
                return;
            }
            // Gesture deltas and `set_default_size` are both in logical
            // (application) pixels: no scale factor here. Only the X11 root
            // window below works in physical pixels.
            let dxl = dx.round() as i32;
            let dyl = dy.round() as i32;
            let (nw, nh, move_x, move_y) = manual_resize_geometry(edge, ow, oh, dxl, dyl);
            win.set_default_size(nw, nh);

            if move_x || move_y {
                if let (Some(origin), Some(x)) = (origin.get(), xid_cell.get()) {
                    let scale = win.scale_factor() as f64;
                    let (nx, ny) =
                        anchored_origin(origin, (ow, oh), (nw, nh), move_x, move_y, scale);
                    x11_pos::move_window(x, nx, ny);
                }
            }
        });
    }

    // Drag end: commit the live allocation as the new default size so the
    // window manager keeps it after the gesture is gone, then clear state.
    {
        let delegated = delegated.clone();
        let area_wk = area.downgrade();
        drag.connect_drag_end(move |_, _, _| {
            delegated.set(false);
            let Some(a) = area_wk.upgrade() else { return };
            let Some(win) = a.root().and_then(|r| r.downcast::<gtk::Window>().ok()) else { return };
            let (w, h) = (win.width(), win.height());
            if w > 0 && h > 0 {
                win.set_default_size(w, h);
            }
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
    /// If true, app follows GNOME system theme automatically (live). Default true for every TontooOS app.
    follow_system: bool,
    glass: Option<(f32, f32, f32)>,
    delegate: Option<Box<dyn AppDelegate>>,
    show_window_bar: bool,
    /// Force the decoration bar on even when the root view draws its own
    /// traffic lights (see [`App::force_window_bar`]). Default false.
    force_window_bar: bool,
    /// If true, the window can be resized (edge handles + resizable). Default true.
    resizable: bool,
    /// If true, oversized content is wrapped in a scroll container. Default true.
    scroll_content: bool,
    /// Device form factor selecting the window corner radius.
    /// Desktop = 26px, Laptop = 24px (matches MacTahoe GTK theme $wm_radius).
    /// Default Desktop so the 26px radius is always set unless laptop mode is enabled.
    form_factor: crate::style::FormFactor,
    /// Window chrome type (Standard = current behavior, Mac = macOS-style
    /// decoration bar). Default Standard so existing apps are unaffected.
    window_type: WindowType,
    /// Window frame (margin, corner radius, border, shadow). Default true.
    /// [`App::no_window_frame`] disables it.
    window_frame: bool,
    /// Window background transparency (1.0 = opaque default).
    /// Below 1.0 the window background becomes semi-transparent (adaptive
    /// rgba: dark 30,30,30 / light 245,245,247) so the wallpaper or windows
    /// behind show through. Real background blur needs compositor support.
    window_alpha: f32,    /// Window backdrop blur radius in px (0.0 = off, default).
    /// Emits `backdrop-filter: blur()` on the window (supported since GTK 4.20;
    /// ignored by older GTK). In-window backdrop content is blurred by GTK
    /// itself; blurring what is *behind* the window additionally needs a
    /// compositor with `ext-background-effect-v1` support.
    window_blur: f32,
    /// If set, the window opens at exactly this size, bypassing the
    /// content-natural-size and half-monitor cap. Default None.
    force_size: Option<(i32, i32)>,
    /// Optional per-frame callback (dt in seconds) driven by a 60 Hz timer.
    tick: Option<Box<dyn FnMut(f32) + Send>>,
    /// Custom title-bar widget (see [`App::set_titlebar_widget`]).
    titlebar_widget: Option<Rc<dyn Widget>>,
    /// Title-bar title visibility (see [`App::show_titlebar_title`]).
    titlebar_show_title: bool,
}

impl App {
    pub fn new(title: impl Into<String>, width: i32, height: i32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
            root: None,
            color_scheme: ColorScheme::Dark,
            follow_system: true,
            glass: None,
            delegate: None,
            show_window_bar: true,
            force_window_bar: false,
            resizable: true,
            scroll_content: true,
            form_factor: crate::style::FormFactor::Desktop,
            window_type: WindowType::Standard,
            window_frame: true,
            window_alpha: 1.0,
            window_blur: 0.0,
            force_size: None,
            tick: None,
            titlebar_widget: None,
            titlebar_show_title: true,
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
        self.follow_system = false;
        CURRENT_SCHEME.store(scheme_to_u8(scheme), Ordering::Relaxed);
    }

    /// Disable the automatic window bar (traffic lights + drag area).
    pub fn no_window_bar(&mut self) -> &mut Self {
        self.show_window_bar = false;
        self
    }

    /// Force the decoration bar on, even when the root view draws its own
    /// traffic lights (e.g. TontooUI `Sidebar`, which hides the bar
    /// automatically). Views then drop their own lights instead — exactly
    /// one set of traffic lights stays visible.
    pub fn force_window_bar(&mut self) -> &mut Self {
        self.show_window_bar = true;
        self.force_window_bar = true;
        self
    }

    /// Disable the window frame: no margin, no corner radius, no border,
    /// no shadow (square content to the screen edge). The title bar (if
    /// enabled) and the resize handles follow `no_window_bar` / `resizable`
    /// as before. Default is framed.
    pub fn no_window_frame(&mut self) -> &mut Self {
        self.window_frame = false;
        self
    }

    /// Custom widget for the title bar: it fills the bar from after the
    /// reserved traffic lights to the right edge (the widget lays out its
    /// own alignment) and replaces the title zone. Rendered on every
    /// rebuild, including the fullscreen reveal bar.
    pub fn set_titlebar_widget(&mut self, widget: impl Widget + 'static) -> &mut Self {
        self.titlebar_widget = Some(Rc::new(widget));
        self
    }

    /// Show or hide the title-bar title text. Default true. The title only
    /// renders when no custom title-bar widget is set.
    pub fn show_titlebar_title(&mut self, show: bool) -> &mut Self {
        self.titlebar_show_title = show;
        self
    }

    /// Lock the window to its initial size: no resize handles and the
    /// window manager is told the window is not resizable. Default is
    /// resizable. Useful for fixed-size cards like About windows.
    pub fn fixed_size(&mut self) -> &mut Self {
        self.resizable = false;
        self
    }

    /// Disable the automatic scroll container. Use only when the content
    /// is guaranteed to fit the window (e.g. fixed-size cards), otherwise
    /// oversized content is clipped instead of scrollable. Default is
    /// scrollable.
    pub fn no_scroll(&mut self) -> &mut Self {
        self.scroll_content = false;
        self
    }

    /// Force the window to open at exactly this size, bypassing the
    /// content-natural-size measurement and the half-monitor cap used by
    /// default. Opt-in only — default is None (automatic sizing).
    pub fn force_size(&mut self, width: i32, height: i32) -> &mut Self {
        self.force_size = Some((width, height));
        self
    }

    /// Auto-detect and set the color scheme from system settings (enables live follow).
    pub fn auto_color_scheme(&mut self) {
        self.color_scheme = ColorScheme::detect_system();
        self.follow_system = true;
        CURRENT_SCHEME.store(scheme_to_u8(self.color_scheme), Ordering::Relaxed);
    }

    /// Force live system-theme follow (default true). Call with false to lock scheme.
    pub fn follow_system_theme(&mut self, follow: bool) -> &mut Self {
        self.follow_system = follow;
        self
    }

    pub fn set_glass(&mut self, milkiness: f32, alpha: f32, sigma: f32) {
        self.glass = Some((milkiness, alpha, sigma));
    }

    pub fn clear_glass(&mut self) {
        self.glass = None;
    }

    /// Set the device form factor (selects window corner radius).
    pub fn set_form_factor(&mut self, form_factor: crate::style::FormFactor) -> &mut Self {
        self.form_factor = form_factor;
        self
    }

    /// Enable laptop mode (24px window corners) or disable it (26px desktop corners).
    pub fn set_laptop_mode(&mut self, laptop: bool) -> &mut Self {
        self.form_factor = if laptop {
            crate::style::FormFactor::Laptop
        } else {
            crate::style::FormFactor::Desktop
        };
        self
    }

    pub fn form_factor(&self) -> crate::style::FormFactor { self.form_factor }

    /// Active window corner radius in px (26 desktop / 24 laptop).
    pub fn window_corner_radius(&self) -> f32 { self.form_factor.window_corner_radius() }

    /// Set window background transparency (`1.0` = opaque default).
    ///
    /// Values below `1.0` make the window background semi-transparent so the
    /// wallpaper or windows behind show through. The color stays adaptive:
    /// dark uses `rgba(30, 30, 30, alpha)`, light uses
    /// `rgba(245, 245, 247, alpha)`. Clamped to `0.05..=1.0`.
    ///
    /// Note: this is transparency only. Live background blur cannot be done
    /// from the client side and needs compositor blur-behind support.
    pub fn set_window_transparency(&mut self, alpha: f32) -> &mut Self {
        self.window_alpha = alpha.clamp(0.05, 1.0);
        self
    }

    pub fn window_alpha(&self) -> f32 { self.window_alpha }

    /// Set the window chrome type (`Standard` default, `Mac` opt-in).
    /// `Standard` keeps current behavior; `Mac` uses the taller macOS-style
    /// decoration bar with centered title.
    pub fn set_window_type(&mut self, window_type: WindowType) -> &mut Self {
        self.window_type = window_type;
        self
    }

    pub fn window_type(&self) -> WindowType { self.window_type }

    /// Apply a glass preset (transparency + blur together).
    /// Default is `GlassStrength::OFF` (opaque, no blur — as before).
    pub fn set_glass_strength(&mut self, strength: crate::style::GlassStrength) -> &mut Self {
        self.set_window_transparency(strength.alpha);
        self.set_window_blur(strength.blur);
        self
    }

    /// Set window backdrop blur radius in px (`0.0` = off, default).
    ///
    /// Emits `backdrop-filter: blur()` on the window background (supported
    /// since GTK 4.20; older GTK ignores the property). Requires a
    /// semi-transparent background (`set_window_transparency` below `1.0`)
    /// to be visible. GTK itself blurs backdrop content inside the window;
    /// blurring what is *behind* the window additionally needs a compositor
    /// with `ext-background-effect-v1` support. Clamped to `0.0..=100.0`.
    pub fn set_window_blur(&mut self, sigma: f32) -> &mut Self {
        self.window_blur = sigma.clamp(0.0, 100.0);
        self
    }

    pub fn window_blur(&self) -> f32 { self.window_blur }

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
        Self::css_for_scheme(self.color_scheme, self.glass, self.window_corner_radius(), self.window_alpha, self.window_blur, self.window_frame)
    }

    fn css_for_scheme(scheme: ColorScheme, glass: Option<(f32, f32, f32)>, window_radius: f32, window_alpha: f32, window_blur: f32, window_frame: bool) -> String {
        let alpha = window_alpha.clamp(0.05, 1.0);
        let (bg_rgba, bg_opaque, fg) = match scheme {
            ColorScheme::Dark => (
                format!("rgba(30, 30, 30, {alpha:.2})"),
                "#1E1E1E",
                "#F5F5F7",
            ),
            ColorScheme::Light => (
                format!("rgba(245, 245, 247, {alpha:.2})"),
                "#F5F5F7",
                "#1E1E1E",
            ),
        };
        // Thin window edge: white in dark mode, black in light mode (like macOS /
        // Parallels card in the reference screenshot), plus outer ring + drop shadow.
        let (edge, inner, outer) = match scheme {
            ColorScheme::Dark => (
                "rgba(255, 255, 255, 0.14)",
                "rgba(255, 255, 255, 0.08)",
                "rgba(0, 0, 0, 0.75)",
            ),
            ColorScheme::Light => (
                "rgba(0, 0, 0, 0.12)",
                "rgba(255, 255, 255, 0.9)",
                "rgba(0, 0, 0, 0.12)",
            ),
        };
        // Backdrop blur (GTK 4.20+). Empty when off so older GTK versions
        // and opaque windows keep byte-identical CSS.
        let blur_css = if window_blur > 0.0 {
            format!("backdrop-filter: blur({:.0}px);", window_blur.clamp(0.0, 100.0))
        } else {
            String::new()
        };
        let mut css = format!(
            "window {{
                background-color: {bg_rgba};
                color: {fg};
                {blur_css}
                border-radius: {window_radius:.0}px;
                border: 1px solid {edge};
                box-shadow: inset 0 1px 0 {inner}, 0 3px 6px rgb(0 0 0 / 15%), 0 7px 24px rgb(0 0 0 / 12%), 0 12px 32px rgb(0 0 0 / 8%), 0 0 0 1px {outer};
                margin: 24px;
                transition: background-color 320ms cubic-bezier(0.32,0.72,0,1), color 320ms cubic-bezier(0.32,0.72,0,1);
            }}
            window:backdrop {{
                box-shadow: inset 0 1px 0 {inner}, 0 3px 6px rgb(0 0 0 / 10%), 0 7px 24px rgb(0 0 0 / 6%), 0 12px 32px transparent, 0 0 0 1px {outer};
            }}
            window decoration {{
                border-radius: {window_radius:.0}px;
                border: 1px solid {edge};
                box-shadow: inset 0 1px 0 {inner}, 0 3px 6px rgb(0 0 0 / 15%), 0 7px 24px rgb(0 0 0 / 12%), 0 12px 32px rgb(0 0 0 / 8%), 0 0 0 1px {outer};
            }}
            window decoration:backdrop {{
                box-shadow: inset 0 1px 0 {inner}, 0 3px 6px rgb(0 0 0 / 10%), 0 7px 24px rgb(0 0 0 / 6%), 0 12px 32px transparent, 0 0 0 1px {outer};
            }}
            window > box, window > overlay {{
                border-radius: {window_radius:.0}px;
            }}
            window.maximized, window.fullscreen {{
                margin: 0;
                border-radius: 0;
                border: none;
                box-shadow: none;
            }}
            window.maximized decoration, window.fullscreen decoration {{
                border-radius: 0;
                border: none;
                box-shadow: none;
            }}
            window.maximized > box, window.maximized > overlay,
            window.fullscreen > box, window.fullscreen > overlay {{
                border-radius: 0;
            }}
            window.maximized .uikit-titlebar, window.fullscreen .uikit-titlebar {{
                border-radius: 0;
            }}
            window.maximized scrolledwindow, window.fullscreen scrolledwindow,
            window.maximized scrolledwindow viewport, window.fullscreen scrolledwindow viewport {{
                border-radius: 0;
            }}
            window.tiled, window.tiled-top, window.tiled-left, window.tiled-right, window.tiled-bottom {{
                margin: 0;
                border-radius: 0;
                box-shadow: none;
            }}
            .uikit-titlebar {{
                background-color: {bg_opaque};
                border-radius: {window_radius:.0}px {window_radius:.0}px 0 0;
                border-bottom: 1px solid {edge};
            }}
            .uikit-titlebar-title {{
                font-family: 'SF Pro Display';
                font-size: 13px;
                font-weight: 600;
            }}
            window * {{
                transition: background-color 320ms cubic-bezier(0.32,0.72,0,1), color 320ms cubic-bezier(0.32,0.72,0,1), background 320ms cubic-bezier(0.32,0.72,0,1);
            }}
            text, label {{
                color: {fg};
                font-family: 'SF Pro Display';
                transition: color 320ms cubic-bezier(0.32,0.72,0,1);
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
                border-radius: {window_radius:.0}px;
                transition: background-color 320ms cubic-bezier(0.32,0.72,0,1);
            }}
            scrolledwindow viewport {{
                background: transparent;
                border-radius: {window_radius:.0}px;
                transition: background-color 320ms cubic-bezier(0.32,0.72,0,1);
            }}",
            bg_rgba = bg_rgba, bg_opaque = bg_opaque, fg = fg,
        );
        if let Some((milkiness, alpha, _sigma)) = glass {
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
        if !window_frame {
            // Frameless override (same specificity, later wins): square
            // content to the screen edge, no margin/border/shadow.
            css.push_str(
                "\n/* no-window-frame */\nwindow { margin: 0; border-radius: 0; border: none; box-shadow: none; }\n\
                 window:backdrop { box-shadow: none; }\n\
                 window decoration { border-radius: 0; border: none; box-shadow: none; }\n\
                 window decoration:backdrop { box-shadow: none; }\n\
                 window > box, window > overlay { border-radius: 0; }\n\
                 .uikit-titlebar { border-radius: 0; }\n\
                 window scrolledwindow, window scrolledwindow viewport { border-radius: 0; }",
            );
        }
        css
    }

    fn setup_live_theme_watcher(provider: gtk::CssProvider, glass: Option<(f32, f32, f32)>, window_radius: f32, window_alpha: f32, window_blur: f32, window_frame: bool) {
        // Watch GTK settings for live GNOME theme changes
        let update = {
            let provider = provider.clone();
            move || {
                let new_scheme = ColorScheme::detect_system();
                let cur = current_color_scheme();
                if cur == Some(new_scheme) {
                    return;
                }
                CURRENT_SCHEME.store(scheme_to_u8(new_scheme), Ordering::Relaxed);
                let new_css = Self::css_for_scheme(new_scheme, glass, window_radius, window_alpha, window_blur, window_frame);
                provider.load_from_string(&new_css);
                // Force redraw of all windows so scrolledwindow/viewport pick up new bg immediately
                if let Some(display) = gtk::gdk::Display::default() {
                    for i in 0..display.monitors().n_items() {
                        let _ = display.monitors().item(i);
                    }
                }
                // Queue draw on active window if any
                APP_STATE.with(|s| {
                    if let Some(ref st) = *s.borrow() {
                        if let Some(ref w) = st.window {
                            w.queue_draw();
                            // Also queue draw on child to ensure viewport updates
                            if let Some(child) = w.child() {
                                child.queue_draw();
                            }
                        }
                    }
                });
            }
        };
        // GTK Settings signals (use local to allow non-Send CssProvider)
        if let Some(settings) = gtk::Settings::default() {
            let upd1 = update.clone();
            settings.connect_notify_local(Some("gtk-theme-name"), move |_, _| upd1());
            let upd2 = update.clone();
            settings.connect_notify_local(
                Some("gtk-application-prefer-dark-theme"),
                move |_, _| upd2(),
            );
        }
        // Poll gsettings color-scheme every 1s as fallback for GNOME portal where GTK notify may not fire
        let mut last = ColorScheme::detect_system();
        glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
            let cur = ColorScheme::detect_system();
            if cur != last {
                last = cur;
                update();
            }
            glib::ControlFlow::Continue
        });
    }

    /// Run the application (blocking event loop).
    pub fn run(&mut self) {
        // Identify this process as a UIKit app for CoreWindows classification.
        mark_toolkit();
        // Fix WSLg rendering: force Wayland socket link + software renderer.
        Self::fix_wslg_environment();

        // TontooOS standard: every app follows GNOME system theme automatically (live, animated)
        // Light bg #F5F5F7 / Dark bg #1E1E1E per user request. If follow_system is true (default),
        // we override the stored scheme with the live system value.
        if self.follow_system {
            self.color_scheme = ColorScheme::detect_system();
        }
        CURRENT_SCHEME.store(scheme_to_u8(self.color_scheme), Ordering::Relaxed);

        let app = Application::builder()
            .application_id("org.tontoo.uikit")
            .flags(gtk::gio::ApplicationFlags::HANDLES_COMMAND_LINE)
            .build();

        let title = self.title.clone();
        let width = self.width;
        let height = self.height;
        let force_size = self.force_size;
        let css = self.build_css();
        let follow_system = self.follow_system;
        let glass_for_watcher = self.glass;
        let radius_for_watcher = self.window_corner_radius();
        let alpha_for_watcher = self.window_alpha;
        let blur_for_watcher = self.window_blur;
        let frame_for_watcher = self.window_frame;
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
                force_window_bar: self.force_window_bar,
                resizable: self.resizable,
                scroll_content: self.scroll_content,
                title: self.title.clone(),
                window_type: self.window_type,
                titlebar: None,
                titlebar_widget: self.titlebar_widget.clone(),
                titlebar_show_title: self.titlebar_show_title,
            });
        });

        // Drive the per-frame tick (defaults to 60 FPS while the app runs).
        if let Some(mut tick) = tick {
            glib::timeout_add_local(Duration::from_millis(1000 / 60), move || {
                tick(1.0 / 60.0);
                glib::ControlFlow::Continue
            });
        }

        // Handle command-line with file args without GIO "can not open files" error.
        // FishPerms prompt is launched as `fishperms-prompt <app> <path>` where <path>
        // is a file to display, not a file to open via GApplication. With HANDLES_COMMAND_LINE
        // we need to explicitly activate.
        app.connect_command_line(|app, _cmdline| {
            app.activate();
            0
        });

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
                let (bar, scroll, resizable, window_type, bar_config) = APP_STATE.with(|s| {
                    s.borrow()
                        .as_ref()
                        .map(|s| (s.show_window_bar, s.scroll_content, s.resizable, s.window_type, TitleBarConfig {
                            custom: s.titlebar_widget.clone(),
                            show_title: s.titlebar_show_title,
                            minimize_enabled: true,
                        }))
                        .unwrap_or((true, true, true, WindowType::Standard, TitleBarConfig {
                            custom: None,
                            show_title: true,
                            minimize_enabled: true,
                        }))
                });
                let (content, nat, _) = build_window_content(view, bar, scroll, &title, window_type, bar_config);
                content_nat = nat;
                // Add invisible edge/corner resize handles around the content.
                if resizable {
                    wrap_window_with_resize_edges(content)
                } else {
                    content
                }
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

            // Allow window resizing even when undecorated (unless fixed-size).
            let resizable = APP_STATE.with(|s| s.borrow().as_ref().map(|s| s.resizable).unwrap_or(true));
            window.set_resizable(resizable);

// Default the window to the requested size, grown to fit the content
            // natural size when larger and capped at half the monitor size in
            // both directions. Oversized content scrolls instead of opening a
            // giant window. The app can still resize afterwards.
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
            if let Some((fw, fh)) = force_size {
                window.set_default_size(fw.max(320), fh.max(240));
            } else if let Some(monitor) = monitor {
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
                let (dw, dh) = resolve_default_size(nat_w, nat_h, width, height, cap_w, cap_h);
                window.set_default_size(dw, dh);
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

            // Backend-only macOS chrome keys: F11 toggles fullscreen in
            // both modes, ESC leaves fullscreen. No public API change.
            let key = gtk::EventControllerKey::new();
            let window_weak = window.downgrade();
            key.connect_key_pressed(move |_, keyval, _, _| {
                let Some(window) = window_weak.upgrade() else {
                    return glib::Propagation::Proceed;
                };
                match fullscreen_key_action(keyval, window.is_fullscreen()) {
                    FullscreenKeyAction::Toggle => {
                        if window.is_fullscreen() {
                            window.unfullscreen();
                        } else {
                            window.fullscreen();
                        }
                        glib::Propagation::Stop
                    }
                    FullscreenKeyAction::Exit => {
                        window.unfullscreen();
                        glib::Propagation::Stop
                    }
                    FullscreenKeyAction::Ignored => glib::Propagation::Proceed,
                }
            });
            window.add_controller(key);

            // Backend-only macOS fullscreen chrome: the bar stays visible
            // in both modes, the traffic lights reveal on top-edge hover.
            // No public API change.
            window.connect_fullscreened_notify(|window| {
                apply_window_chrome(window.is_fullscreen());
            });
            let motion = gtk::EventControllerMotion::new();
            motion.connect_motion(|_, _x, y| {
                APP_STATE.with(|state| {
                    if let Some(app) = state.borrow().as_ref() {
                        if let (Some(window), Some(bar)) =
                            (app.window.clone(), app.titlebar.clone())
                        {
                            if window.is_fullscreen() {
                                set_bar_lights_visible(&bar, fullscreen_reveal_lights(y));
                            }
                        }
                    }
                });
            });
            motion.connect_leave(|_| {
                APP_STATE.with(|state| {
                    if let Some(app) = state.borrow().as_ref() {
                        if let (Some(window), Some(bar)) =
                            (app.window.clone(), app.titlebar.clone())
                        {
                            if window.is_fullscreen() {
                                set_bar_lights_visible(&bar, false);
                            }
                        }
                    }
                });
            });
            window.add_controller(motion);

            // Live GNOME theme watcher — updates CSS with animation on system change
            if follow_system {
                Self::setup_live_theme_watcher(css_provider.clone(), glass_for_watcher, radius_for_watcher, alpha_for_watcher, blur_for_watcher, frame_for_watcher);
            }

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

    #[test]
    fn app_fixed_size_no_scroll() {
        let app = App::new("Test", 800, 600);
        assert!(app.resizable);
        assert!(app.scroll_content);
        let mut fixed = App::new("Card", 320, 580);
        fixed.fixed_size();
        fixed.no_scroll();
        assert!(!fixed.resizable);
        assert!(!fixed.scroll_content);
    }

    #[test]
    fn effective_window_bar_rules() {
        // Forced bar always wins, even with a sidebar.
        assert!(effective_window_bar(true, true, true));
        assert!(effective_window_bar(false, true, true));
        // A hiding tree switches an enabled bar off.
        assert!(!effective_window_bar(true, false, true));
        // Without a hiding tree the user choice stands.
        assert!(effective_window_bar(true, false, false));
        assert!(!effective_window_bar(false, false, false));
    }

    #[test]
    fn hides_window_bar_recursive_walks_children() {
        use crate::widget::WidgetId;

        struct Plain;
        impl Widget for Plain {
            fn id(&self) -> WidgetId {
                0
            }
            fn to_gtk(&self) -> gtk::Widget {
                unimplemented!()
            }
        }

        struct Hider;
        impl Widget for Hider {
            fn id(&self) -> WidgetId {
                1
            }
            fn to_gtk(&self) -> gtk::Widget {
                unimplemented!()
            }
            fn hides_window_bar(&self) -> bool {
                true
            }
        }

        struct Parent {
            child: Box<dyn Widget>,
        }
        impl Widget for Parent {
            fn id(&self) -> WidgetId {
                2
            }
            fn to_gtk(&self) -> gtk::Widget {
                unimplemented!()
            }
            fn children(&self) -> Vec<&dyn Widget> {
                vec![self.child.as_ref()]
            }
        }

        assert!(!Plain.hides_window_bar_recursive());
        assert!(Hider.hides_window_bar_recursive());
        assert!(Parent {
            child: Box::new(Hider)
        }
        .hides_window_bar_recursive());
        assert!(!Parent {
            child: Box::new(Plain)
        }
        .hides_window_bar_recursive());
    }

    #[test]
    fn force_window_bar_opt_in() {
        let mut app = App::new("Test", 800, 600);
        assert!(!app.force_window_bar);
        app.force_window_bar();
        assert!(app.force_window_bar);
        assert!(app.show_window_bar);
    }

    #[test]
    fn app_force_size_opt_in() {
        let app = App::new("Test", 800, 600);
        assert_eq!(app.force_size, None);
        let mut big = App::new("Big", 800, 600);
        big.force_size(820, 1840);
        assert_eq!(big.force_size, Some((820, 1840)));
    }

    #[test]
    fn app_window_radius_defaults_to_desktop() {
        let app = App::new("Test", 800, 600);
        assert_eq!(app.form_factor(), crate::style::FormFactor::Desktop);
        assert_eq!(app.window_corner_radius(), 26.0);
        let css = app.build_css();
        assert!(css.contains("border-radius: 26px"));
        assert!(css.contains("window decoration"));
        assert!(css.contains("box-shadow: inset 0 1px 0"));
        // Dark mode gets a thin white edge.
        assert!(css.contains("border: 1px solid rgba(255, 255, 255, 0.14)"));
    }

    #[test]
    fn app_window_radius_laptop_mode() {
        let mut app = App::new("Test", 800, 600);
        app.set_laptop_mode(true);
        assert_eq!(app.form_factor(), crate::style::FormFactor::Laptop);
        assert_eq!(app.window_corner_radius(), 24.0);
        let css = app.build_css();
        assert!(css.contains("border-radius: 24px"));
        assert!(css.contains("box-shadow: inset 0 1px 0"));
    }

    #[test]
    fn app_window_edge_light_mode() {
        let mut app = App::new("Test", 800, 600);
        app.set_color_scheme(ColorScheme::Light);
        let css = app.build_css();
        // Light mode gets a thin black edge.
        assert!(css.contains("border: 1px solid rgba(0, 0, 0, 0.12)"));
    }

    #[test]
    fn app_window_opaque_by_default() {
        let app = App::new("Test", 800, 600);
        assert_eq!(app.window_alpha(), 1.0);
        assert!(app.build_css().contains("rgba(30, 30, 30, 1.00)"));
    }

    #[test]
    fn app_window_transparency_adaptive() {
        let mut app = App::new("Test", 800, 600);
        app.set_window_transparency(0.8);
        assert_eq!(app.window_alpha(), 0.8);
        assert!(app.build_css().contains("rgba(30, 30, 30, 0.80)"));

        app.set_color_scheme(ColorScheme::Light);
        assert!(app.build_css().contains("rgba(245, 245, 247, 0.80)"));
    }

    #[test]
    fn app_window_transparency_clamped() {
        let mut app = App::new("Test", 800, 600);
        app.set_window_transparency(5.0);
        assert_eq!(app.window_alpha(), 1.0);
        app.set_window_transparency(-1.0);
        assert_eq!(app.window_alpha(), 0.05);
    }

    #[test]
    fn app_window_blur_off_by_default() {
        let app = App::new("Test", 800, 600);
        assert_eq!(app.window_blur(), 0.0);
        assert!(!app.build_css().contains("backdrop-filter"));
    }

    #[test]
    fn app_window_blur_emits_css() {
        let mut app = App::new("Test", 800, 600);
        app.set_window_transparency(0.85);
        app.set_window_blur(20.0);
        assert_eq!(app.window_blur(), 20.0);
        assert!(app.build_css().contains("backdrop-filter: blur(20px)"));
    }

    #[test]
    fn app_window_blur_clamped() {
        let mut app = App::new("Test", 800, 600);
        app.set_window_blur(500.0);
        assert_eq!(app.window_blur(), 100.0);
        app.set_window_blur(-5.0);
        assert_eq!(app.window_blur(), 0.0);
    }

    #[test]
    fn app_window_frame_on_by_default() {
        let css = App::new("Test", 800, 600).build_css();
        assert!(!css.contains("no-window-frame"));
        assert!(css.contains("margin: 24px;"));
    }

    #[test]
    fn app_no_window_frame_override() {
        let mut app = App::new("Test", 800, 600);
        app.no_window_frame();
        let css = app.build_css();
        // Override block present (same specificity, later wins).
        assert!(css.contains("/* no-window-frame */"));
        assert!(css.contains(".uikit-titlebar { border-radius: 0; }"));
        assert!(css.contains("window > box, window > overlay { border-radius: 0; }"));
    }

    #[test]
    fn app_titlebar_is_opaque() {
        let app = App::new("Test", 800, 600);
        let css = app.build_css();
        assert!(css.contains(".uikit-titlebar"));
        assert!(css.contains(".uikit-titlebar-title"));
        // Dark title bar is solid, not transparent.
        assert!(css.contains("background-color: #1E1E1E"));

        let mut light = App::new("Test", 800, 600);
        light.set_color_scheme(ColorScheme::Light);
        assert!(light.build_css().contains("background-color: #F5F5F7"));
    }

    #[test]
    fn app_fullscreen_has_no_margin_or_radius() {
        // Fullscreen/maximized/tiled windows fill the screen: no shadow
        // margin, no rounded corners, no shadow.
        let css = App::new("Test", 800, 600).build_css();
        assert!(css.contains("window.maximized, window.fullscreen"));
        assert!(css.contains("window.tiled"));
        assert!(css.contains("margin: 0;"));
    }

    #[test]
    fn app_content_single_layer_background() {
        // Only the window itself carries the alpha background; scrolled
        // containers stay transparent so no stacked darker box appears.
        let mut app = App::new("Test", 800, 600);
        app.set_window_transparency(0.85);
        let css = app.build_css();
        assert!(css.contains("rgba(30, 30, 30, 0.85)"));
        assert!(css.contains("scrolledwindow"));
        assert!(!css.contains("scrolledwindow viewport {\n                background: rgba"));
    }

    #[test]
    fn default_size_honors_requested() {
        // Requested size is the base.
        assert_eq!(resolve_default_size(100, 100, 640, 480, 960, 540), (640, 480));
        // Grows to fit larger content, capped at half the monitor.
        assert_eq!(resolve_default_size(800, 500, 640, 480, 960, 540), (800, 500));
        assert_eq!(resolve_default_size(2000, 2000, 640, 480, 960, 540), (960, 540));
        // Never below minimum window size.
        assert_eq!(resolve_default_size(0, 0, 100, 100, 960, 540), (320, 240));
    }

    #[test]
    fn window_type_defaults_to_standard() {
        let app = App::new("Test", 800, 600);
        assert_eq!(app.window_type(), WindowType::Standard);
        assert_eq!(WindowType::Standard.title_bar_height(), 31);
        assert_eq!(WindowType::Mac.title_bar_height(), 44);
        // Standard CSS is unchanged (opaque, no blur).
        assert!(!app.build_css().contains("backdrop-filter"));
    }

    #[test]
    fn window_type_mac_opt_in() {
        let mut app = App::new("Test", 800, 600);
        app.set_window_type(WindowType::Mac);
        assert_eq!(app.window_type(), WindowType::Mac);
    }

    #[test]
    fn glass_strength_applies_alpha_and_blur() {
        let mut app = App::new("Test", 800, 600);
        app.set_glass_strength(crate::style::GlassStrength::BALANCED);
        assert_eq!(app.window_alpha(), 0.85);
        assert_eq!(app.window_blur(), 20.0);
        assert!(app.build_css().contains("backdrop-filter: blur(20px)"));
    }

    #[test]
    fn fullscreen_keys_toggle_and_exit() {
        // F11 toggles in both modes, ESC only leaves fullscreen.
        use gtk::gdk::Key;
        assert_eq!(fullscreen_key_action(Key::F11, false), FullscreenKeyAction::Toggle);
        assert_eq!(fullscreen_key_action(Key::F11, true), FullscreenKeyAction::Toggle);
        assert_eq!(fullscreen_key_action(Key::Escape, true), FullscreenKeyAction::Exit);
        assert_eq!(fullscreen_key_action(Key::Escape, false), FullscreenKeyAction::Ignored);
        assert_eq!(fullscreen_key_action(Key::Return, true), FullscreenKeyAction::Ignored);
        assert_eq!(fullscreen_key_action(Key::Return, false), FullscreenKeyAction::Ignored);
    }

    #[test]
    fn fullscreen_reveal_strip() {
        // The traffic lights show while the pointer touches the top edge.
        assert!(fullscreen_reveal_lights(0.0));
        assert!(fullscreen_reveal_lights(31.9));
        assert!(!fullscreen_reveal_lights(32.0));
        assert!(!fullscreen_reveal_lights(400.0));
    }

    #[test]
    fn manual_resize_geometry_edges() {
        // East grows to the right, no window move.
        assert_eq!(
            manual_resize_geometry(ResizeEdge::East, 800, 600, 50, 0),
            (850, 600, false, false)
        );
        // South grows downward, no window move.
        assert_eq!(
            manual_resize_geometry(ResizeEdge::South, 800, 600, 0, 40),
            (800, 640, false, false)
        );
        // West grows to the left and moves the window; height untouched.
        assert_eq!(
            manual_resize_geometry(ResizeEdge::West, 800, 600, -50, 99),
            (850, 600, true, false)
        );
        // North grows upward and moves the window; width untouched.
        assert_eq!(
            manual_resize_geometry(ResizeEdge::North, 800, 600, 99, -30),
            (800, 630, false, true)
        );
        // SouthEast grows in both directions without moving.
        assert_eq!(
            manual_resize_geometry(ResizeEdge::SouthEast, 800, 600, 10, 20),
            (810, 620, false, false)
        );
        // NorthWest grows in both directions and moves in both axes.
        assert_eq!(
            manual_resize_geometry(ResizeEdge::NorthWest, 800, 600, -10, -20),
            (810, 620, true, true)
        );
        // Shrinking past the minimum clamps instead of collapsing (east
        // ignores the vertical delta, so the height is untouched).
        assert_eq!(
            manual_resize_geometry(ResizeEdge::East, 330, 250, -500, -500),
            (320, 250, false, false)
        );
        assert_eq!(
            manual_resize_geometry(ResizeEdge::SouthEast, 330, 250, -500, -500),
            (320, 240, false, false)
        );
    }

    #[test]
    fn anchored_origin_keeps_opposite_edge() {
        // West drag left by 50 logical px at scale 1: window moves left.
        assert_eq!(
            anchored_origin((100, 200), (800, 600), (850, 600), true, false, 1.0),
            (50, 200)
        );
        // HiDPI scale 2 converts the logical move to physical pixels.
        assert_eq!(
            anchored_origin((100, 200), (800, 600), (850, 600), true, false, 2.0),
            (0, 200)
        );
        // Clamped at the minimum: the window only moves by the size change
        // that was actually applied, so the right edge stays anchored.
        assert_eq!(
            anchored_origin((100, 200), (330, 600), (320, 600), true, false, 1.0),
            (110, 200)
        );
        // East/south drags never move the window.
        assert_eq!(
            anchored_origin((100, 200), (800, 600), (850, 640), false, false, 1.0),
            (100, 200)
        );
    }
}
