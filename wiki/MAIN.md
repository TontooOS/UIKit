# TontooUIKit -- Wiki

TontooUIKit is a native Rust UI toolkit for TontooOS, built on GTK4. It follows Apple's UIKit design philosophy with a `View` base class, constraint-based layout, view controllers, and a built-in shader system for GPU-accelerated effects.

- Repository: tontoo-os/TontooLibs/UIKit
- License: MIT
- Version: 26.1.0

## Feature Index

| Feature | File | Description |
|---|---|---|
| Main index | [MAIN.md](MAIN.md) | This page |
| Rules | [RULE.md](RULE.md) | Development and usage rules |
| View | [View.md](View.md) | View base class, subviews, rendering |
| Style | [Style.md](Style.md) | Design tokens, window corner radii (26px desktop / 24px laptop) |
| Widgets | [Widgets.md](Widgets.md) | Pre-built UI elements (Label, Button, Toggle, etc.) |
| Shader | [Shader.md](Shader.md) | GLSL shader loading, built-in effects, ShaderView |
| ViewController | [ViewController.md](ViewController.md) | View controllers, navigation, lifecycle |
| Constraints | [Constraints.md](Constraints.md) | Auto Layout constraint system |

## Fullscreen / Maximize

TontooUIKit windows are undecorated by default and use a custom title bar with
macOS-style traffic lights. The green button toggles between fullscreen and
windowed mode using `window.fullscreen()` / `window.unfullscreen()` (the
`__maximize` action in `dispatch_custom`). The red button closes the window
(`__close`), the yellow button minimizes it (`__minimize`).

Backend-only macOS fullscreen chrome (no public API change, all in
`App::run` / `apply_window_chrome`):

- **F11** toggles fullscreen in both modes, **ESC** leaves fullscreen
  (`FullscreenKeyAction`, wired via a window `EventControllerKey`).
- The decoration bar stays visible in both modes with identical button
  positions; entering fullscreen only drops the invisible resize edges.
- The traffic lights hide in fullscreen (keeping layout space via opacity,
  so nothing shifts) and reveal while the pointer touches the top 32 px
  edge, like the macOS menu-bar reveal.
- Revealed lights keep close (works) and green (exits fullscreen through
  the regular `__maximize` toggle); the middle minimize button is disabled
  (stays gray, ignores clicks) via `TrafficLights::minimize_enabled(false)`.
- Leaving fullscreen (green button, F11, ESC or compositor) restores edges,
  lights and minimize.
- `dispatch_custom` rebuilds through the same path, so delegate updates
  never leak windowed chrome into fullscreen or vice versa.

## Window Resizing

Undecorated windows get no resize borders from the compositor, so `App::run`
wraps the window content in a `gtk::Overlay` with eight invisible resize
handles: one strip on each edge (`North`, `South`, `West`, `East`, 6 px) and
one square on each corner (`NorthWest`, `NorthEast`, `SouthWest`,
`SouthEast`, 14 px). Corners are added last so they sit on top of the edge
strips.

Each handle shows the matching resize cursor (`n-resize`, `se-resize`, ...).
A press starts a `gtk::GestureDrag`. Exactly one backend owns the resize, so
the window keeps its size after the button is released instead of snapping
back:

- **Native Wayland**: `gdk_toplevel_begin_resize` is called synchronously in
  the press handler with surface-local press coordinates (the handle-local
  press point translated into the window) and the press device (so GDK passes
  the matching implicit-grab serial). The compositor then runs the whole
  resize with correct edge anchoring on all sides, and the manual fallback
  stays off for the rest of the gesture. This is the path TontooCompositor
  serves.
- **X11 / XWayland (WSLg)**: the gesture resizes manually via
  `set_default_size` in logical pixels (gesture deltas and default size share
  the same unit, so no scale factor is applied); west/north edges
  additionally move the window through X11 (`x11rb`, position-only configure)
  by the size change that was actually applied after min-size clamping, so the
  opposite edge stays anchored. No compositor handoff is sent, so no
  window-manager session can revert the manual size on release. On release the
  live allocation is committed once more as the default size.

The handles are re-applied by `dispatch_custom` after every delegate rebuild,
so resizing survives state updates. Set `UIKIT_RESIZE_DEBUG=1` to paint the
handles in debug colors and log their allocations.

## Auto-scroll, screen fit and split layout

`App::run` wraps the view so content scrolls instead of growing the window. If
the root is an `HStack` with exactly two children, it is treated as a window
layout: the left child becomes a fixed sidebar that fills the full window
height (only its own internal content may scroll), and the right child becomes
the scrollable content area. Any other root is wrapped in a single scroll
container. With the window bar enabled, only the content scrolls; the
traffic-light bar stays fixed. The window defaults to the requested size
(`App::new` width/height, including the title bar height), grown to fit the
content's natural size when larger and capped at half the monitor size in
both directions (the content is
measured before wrapping, the monitor size is divided by its scale factor so
the cap is half the visible screen). Delegate rebuilds via `dispatch_custom`
re-apply the same layout and the resize handles, so scroll protection and
resizing never disappear after an interaction.

## Quick Start

```rust
use uikit::prelude::*;

fn main() {
    let mut app = App::new("My App", 800, 600);
    app.set_color_scheme(ColorScheme::Dark);
    app.set_delegate(MyApp);
    app.run();
}

struct MyApp;

impl AppDelegate for MyApp {
    fn view(&self) -> Box<dyn Widget> {
        let mut root = View::empty();
        root.set_frame(0.0, 0.0, 800.0, 600.0);

        let title = Label::new("Hello TontooOS")
            .font_size(24.0)
            .bold()
            .color(Color::WHITE);
        root.add_subview(View::new(title).with_frame(16.0, 16.0, 300.0, 36.0));

        let btn = Button::new("Click Me")
            .on_click(|| println!("Clicked!"));
        root.add_subview(View::new(btn).with_frame(16.0, 60.0, 120.0, 44.0));

        struct W(View);
        impl Widget for W {
            fn id(&self) -> uikit::widget::WidgetId { 0 }
            fn to_gtk(&self) -> gtk::Widget { self.0.to_gtk_scrollable() }
        }
        Box::new(W(root))
    }
}
```

See [View.md](View.md), [Widgets.md](Widgets.md), and [Shader.md](Shader.md) for details.

## Architecture

```
App (event loop, window, CSS)
 |
 +-- AppDelegate (trait: view() -> Box<dyn Widget>)
 |
 +-- View (base class)
 |    +-- subviews: Vec<View>
 |    +-- frame: Rect
 |    +-- content: Box<dyn ViewContent>
 |    +-- to_gtk() / to_gtk_scrollable()
 |
 +-- Widgets (Label, Button, TextField, etc.)
 |    +-- implement ViewContent
 |    +-- implement Widget (backward compat)
 |
 +-- ShaderView (GLArea + GLSL shaders)
 |    +-- Shader (compile, uniforms)
 |    +-- Built-in: gradient, blur, glass, noise, rainbow, wave
 |
 +-- ViewController (lifecycle, presentation)
 +-- Constraints (Auto Layout)
```

## Cross References

- [View.md](View.md) -- base class for all UI elements
- [Style.md](Style.md) -- design tokens and window corner radii
- [Widgets.md](Widgets.md) -- pre-built widget catalog
- [Shader.md](Shader.md) -- GPU shader system
- [ViewController.md](ViewController.md) -- screen management
- [Constraints.md](Constraints.md) -- layout system

## Toolkit Identity

Every UIKit app publishes its toolkit via the `TONTOO_TOOLKIT=UIKit`
environment variable (set automatically by `App::run`, or manually with
`uikit::app::mark_toolkit()` for custom event loops). CoreWindows reads it
from `/proc/<pid>/environ` to classify open windows. The constants
`TOOLKIT_ENV_VAR` / `TOOLKIT_ID` are re-exported in the prelude.

## Performance Notes

TontooUIKit is designed so long-running apps do not accumulate work or memory
over time:

- `apply_css` attaches a `CssProvider` to the widget's own style context. The
  provider cascades to the widget's descendants and is released together with
  the widget when it is destroyed. Providers are never registered on the
  display, which would keep them alive for the whole process and slow CSS
  matching down the longer the app runs.
- Animated widgets cache their scale-transform `CssProvider` and skip frames
  where position, opacity and scale did not change, so the always-running
  60 Hz tick does not force a full repaint of idle widgets.
- `ShaderView` drives auto-animation with a self-cancelling timer that only
  holds a weak reference to the `GLArea`. The timer stops itself once the
  widget is destroyed instead of piling up 60 FPS redraw sources.
- `TrafficLights` connects the window `is-active` watcher exactly once per
  instance, so repeated minimize/restore cycles do not stack signal handlers.
- The shader fullscreen quad (VAO/VBO) is created once and reused across
  frames. `ZStack` renders the first child only once (as the main child, not
  as an overlay).

## Changelog

- 2026-09-10: Window resize keeps its size on release (Wayland delegates
  fully with surface-local coordinates, X11 resizes purely manually, drag end
  commits the live allocation).
- 2026-09-08: Toolkit identity (`TONTOO_TOOLKIT=UIKit` via `App::run` /
  `mark_toolkit`, `TOOLKIT_ENV_VAR` / `TOOLKIT_ID` in the prelude) so
  CoreWindows can classify UIKit windows.
