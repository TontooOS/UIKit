//! # Glass Window Demo
//!
//! Test app for window transparency, corner radius and edge highlight.
//!
//! Run with: `cargo run --example glass_window`
//!
//! Environment (all optional):
//!
//! - `UIKIT_DEMO_ALPHA` — window transparency, `0.05..=1.0` (default `0.85`).
//!   Below `1.0` the wallpaper or windows behind show through the window
//!   background.
//! - `UIKIT_DEMO_BLUR` — backdrop blur radius in px, `0.0..=100.0` (default `20.0`).
//!   Needs GTK 4.20+ (checked: WSL ships 4.22.4). Blurring what is behind the
//!   window additionally needs a compositor with `ext-background-effect-v1`.
//! - `UIKIT_DEMO_SCHEME` — `dark` or `light` to lock the scheme, or unset to
//!   follow the system live (toggle via ini/tontoo theme files is picked up
//!   within ~1s through the GTK settings watcher).
//! - `UIKIT_DEMO_LAPTOP` — `1` for laptop corners (24px), default desktop (26px).
//! - `UIKIT_DEMO_TYPE` — `mac` for the macOS-style window (taller decoration
//!   bar), default standard window. No API change for existing apps.
//!
//! ```bash
//! UIKIT_DEMO_ALPHA=0.7 UIKIT_DEMO_BLUR=30 UIKIT_DEMO_SCHEME=light cargo run --example glass_window
//! UIKIT_DEMO_TYPE=mac cargo run --example glass_window
//! ```
//!
//! The demo also shows example title-bar buttons (Refresh/Share/Info via
//! `App::set_titlebar_widget`) and supports F11 (toggle fullscreen) and
//! ESC (leave fullscreen) like every UIKit app.
//!
//! Note: the in-window text colors are a startup snapshot; the window
//! background, edge and blur follow live system toggles.

use uikit::prelude::*;

fn env_alpha() -> f32 {
    std::env::var("UIKIT_DEMO_ALPHA")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(0.85)
        .clamp(0.05, 1.0)
}

fn env_blur() -> f32 {
    std::env::var("UIKIT_DEMO_BLUR")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(20.0)
        .clamp(0.0, 100.0)
}

fn env_scheme() -> Option<bool> {
    match std::env::var("UIKIT_DEMO_SCHEME")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "light" => Some(false),
        "dark" => Some(true),
        _ => None,
    }
}

fn env_laptop() -> bool {
    matches!(
        std::env::var("UIKIT_DEMO_LAPTOP").unwrap_or_default().as_str(),
        "1" | "true" | "yes"
    )
}

fn env_mac() -> bool {
    matches!(
        std::env::var("UIKIT_DEMO_TYPE")
            .unwrap_or_default()
            .to_lowercase()
            .as_str(),
        "mac" | "macos"
    )
}

struct GlassDemo {
    alpha: f32,
    blur: f32,
    dark: bool,
    laptop: bool,
    mac: bool,
}

impl AppDelegate for GlassDemo {
    fn view(&self) -> Box<dyn Widget> {
        let scheme = if self.dark { "Dark" } else { "Light" };
        let corners = if self.laptop {
            "Laptop corners (24px)"
        } else {
            "Desktop corners (26px)"
        };
        let wtype = if self.mac { "Mac window" } else { "Standard window" };
        let fg = if self.dark {
            Color::new(0.96, 0.96, 0.97, 1.0)
        } else {
            Color::new(0.12, 0.12, 0.12, 1.0)
        };
        let dim = if self.dark {
            Color::new(0.70, 0.70, 0.72, 1.0)
        } else {
            Color::new(0.42, 0.42, 0.44, 1.0)
        };
        Box::new(
            VStack::new()
                .spacing(10.0)
                .child(Text::new("Glass Window Demo").font_size(28.0).bold().color(fg))
                .child(
                    Text::new(format!(
                        "Alpha {:.2} - Blur {:.0}px - {} - {} - {}",
                        self.alpha, self.blur, scheme, corners, wtype
                    ))
                    .font_size(14.0)
                    .color(dim),
                )
                .child(
                    Text::new("Move a window or the wallpaper behind this window to test transparency.")
                        .font_size(13.0)
                        .color(dim),
                )
                .child(
                    Text::new("Backdrop blur needs GTK 4.20+ plus a compositor with ext-background-effect-v1.")
                        .font_size(13.0)
                        .color(dim),
                )
                .child(Button::new("Close").on_custom("__close")),
        )
    }
}

fn main() {
    let alpha = env_alpha();
    let blur = env_blur();
    let laptop = env_laptop();
    let mac = env_mac();
    // Locked scheme via env, otherwise follow the system live (startup
    // snapshot decides the in-window text colors).
    let locked_dark = env_scheme();
    let dark = locked_dark.unwrap_or_else(|| ColorScheme::detect_system() == ColorScheme::Dark);

    println!(
        "GlassWindow demo: alpha={:.2} blur={:.0}px scheme={} corners={} type={}",
        alpha,
        blur,
        if dark { "dark" } else { "light" },
        if laptop { "laptop/24px" } else { "desktop/26px" },
        if mac { "mac" } else { "standard" }
    );

    let delegate = GlassDemo { alpha, blur, dark, laptop, mac };
    let mut app = App::with_delegate("Glass Window Demo", 640, 480, delegate);
    // Example title-bar content: buttons filling the bar after the
    // reserved traffic lights (left) to the right edge.
    app.set_titlebar_widget(
        HStack::new()
            .spacing(8.0)
            .child(Button::new("Refresh").on_click(|| println!("titlebar: Refresh clicked")))
            .child(Button::new("Share").on_click(|| println!("titlebar: Share clicked")))
            .child(Button::new("Info").on_click(|| println!("titlebar: Info clicked"))),
    );
    if mac {
        app.set_window_type(WindowType::Mac);
    }
    app.set_window_transparency(alpha);
    app.set_window_blur(blur);
    app.set_laptop_mode(laptop);
    match locked_dark {
        Some(true) => app.set_color_scheme(ColorScheme::Dark),
        Some(false) => app.set_color_scheme(ColorScheme::Light),
        None => app.auto_color_scheme(),
    }
    app.run();
}
