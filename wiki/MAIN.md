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
- [Widgets.md](Widgets.md) -- pre-built widget catalog
- [Shader.md](Shader.md) -- GPU shader system
- [ViewController.md](ViewController.md) -- screen management
- [Constraints.md](Constraints.md) -- layout system
