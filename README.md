# TontooUIKit

**Lightweight, native Rust UI toolkit for TontooOS.**

TontooUIKit is the foundation for building native applications on TontooOS.
Apps built with TontooUIKit are extremely lightweight — they describe their
UI as a declarative widget tree and the compositor renders everything
server-side. No GPU, no renderer, no external UI dependencies.

## Architecture

```
┌─────────────┐     widget tree      ┌──────────────────┐
│  TontooApp   │ ──────────────────►  │  TontooCompositor │
│ (client)     │ ◄────────────────── │  (server)         │
└─────────────┘   events (click,      └──────────────────┘
                    hover, key)         renders with
                                        glass, shadows,
                                        rounded corners
```

TontooUIKit communicates with the TontooOS compositor via the **tontoo_ui.xml
Wayland protocol**. Apps send serialized widget trees through the
`update_widget_tree` request. The compositor sends back events (clicks, hovers,
keyboard input) through `widget_clicked`, `widget_hovered`, and `key_event`.

### Why server-side rendering?

- **Consistent appearance**: The compositor controls the look — liquid glass,
  shadows, rounded corners, typography. All apps look macOS-like.
- **Minimal client overhead**: Apps don't need a GPU or rendering library.
  They just describe their UI in a declarative widget tree.
- **Security**: The compositor mediates all rendering; apps can't draw
  arbitrary pixels or read other apps' content.

## Widgets

| Widget | Description |
|--------|-------------|
| `Text` | Read-only text display |
| `Button` | Clickable label with glass or solid background |
| `Image` | Bitmap image by file path |
| `Toggle` | On/off switch (pill-shaped track + thumb) |
| `TextField` | Single-line text input field |
| `Divider` | Horizontal or vertical line separator |
| `Spacer` | Flexible or fixed empty space |
| `Card` | Container with glass-panel or solid background |

## Layout

| Layout | Description |
|--------|-------------|
| `VStack` | Vertical stack (top-to-bottom) |
| `HStack` | Horizontal stack (left-to-right) |
| `ZStack` | Layered stack (back-to-front) |
| `PaddingWrap` | Adds spacing around a child |
| `Frame` | Fixed or bounded size container |

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
tontoo-uikit = { path = "../TontooLibs/UIKit" }
```

### AppDelegate pattern (recommended)

```rust
use tontoo_uikit::prelude::*;

struct MyApp { count: u32 }

impl AppDelegate for MyApp {
    fn view(&self) -> Box<dyn Widget> {
        Box::new(
            VStack::new()
                .child(Text::new(format!("Count: {}", self.count)).font_size(48.0))
                .child(
                    HStack::new()
                        .spacing(12.0)
                        .child(Button::new("+").on_custom("inc"))
                        .child(Button::new("-").on_custom("dec")),
                )
        )
    }

    fn handle_custom(&mut self, action: &str) {
        match action {
            "inc" => self.count += 1,
            "dec" => self.count = self.count.saturating_sub(1),
            _ => {}
        }
    }
}

fn main() {
    let mut app = App::with_delegate("My App", 400, 300, MyApp { count: 0 });
    app.set_glass(0.25, 0.65, 20.0);
    app.run();
}
```

### Low-level widget tree pattern

```rust
use tontoo_uikit::prelude::*;

fn main() {
    let root = VStack::new()
        .child(Text::new("Hello, TontooOS!").font_size(24.0).bold())
        .child(Spacer::new())
        .child(
            Button::new("Click Me")
                .on_click(|| println!("Clicked!")),
        );

    let mut app = App::new("My App", 800, 600);
    app.set_glass(0.25, 0.65, 20.0);
    app.set_root(root);
    app.run();
}
```

## Serialization

The widget tree is serialized into a compact binary format for the
`tontoo_ui` Wayland protocol:

```
[u32 node_count]
For each node:
  [u8  type_tag]         WidgetType discriminant (1–14)
  [var  properties]      Type-specific fields
  [u32 child_count]      Number of children
  [u32 child_id × N]     Array indices of child nodes
```

Use `serialization::serialize_widget_tree()` to produce the bytes, or
`serialization::flatten_tree()` to convert a live widget tree into
`FlatWidget` nodes first.

## DrawCommand

The `DrawCommand` enum mirrors the compositor's `widget_renderer::DrawCommand`:

```rust
pub enum DrawCommand {
    Text { content, x, y, font_size, color, max_width },
    Rect { x, y, width, height, color, corner_radius },
    GlassPanel { x, y, width, height, milkiness, alpha, corner_radius },
    Image { x, y, width, height, path },
}
```

## Event System

Events are routed from the compositor through the `tontoo_ui` protocol:

- **Click** — `widget_clicked` → `Event::Click(ClickEvent { position, button })`
- **Hover** — `widget_hovered` → `Event::Hover(HoverEvent { position })`
- **Key** — `key_event` → `Event::Key(KeyEvent { key, pressed, modifiers })`
- **Focus** — managed by the runtime; `Action::Focus(id)` / `Action::Unfocus`

Widgets return `Option<Action>` from `handle_event()` to signal side effects.

## Version Scheme

```rust
pub const UITKIT_VERSION: (u32, u32, u32) = (0, 1, 0);
```

Apps can check the version at compile time:

```rust
if tontoo_uikit::UITKIT_VERSION.1 >= 1 {
    // Use features from 0.1.0+
}
```

## Running the Example

```bash
cargo run --example hello
```

## Building for TontooOS

The library targets `x86_64-unknown-linux-gnu`. Cross-compile with:

```bash
rustup target add x86_64-unknown-linux-gnu
cargo build --target x86_64-unknown-linux-gnu --release
```

## License

MIT
