# Widgets

Pre-built UI elements provided by TontooUIKit. Each widget implements `ViewContent` for use with `View::new()` and `Widget` for backward compatibility.

## Widget Catalog

| Widget | Description | GTK Backend |
|---|---|---|
| `Label` / `Text` | Text display | `gtk::Label` |
| `Button` | Clickable button | `gtk::Button` |
| `TextField` | Text input | `gtk::Entry` |
| `ImageView` / `Image` | Image display | `gtk::Picture` |
| `Toggle` | On/off switch | `gtk::Switch` |
| `Separator` | Divider line | `gtk::Separator` |
| `Icon` | Icon display | `gtk::Picture` |
| `ScrollView` | Scrollable container | `gtk::ScrolledWindow` |
| `ListView` | List of items | `gtk::ScrolledWindow` |
| `TrafficLights` | Window controls | `gtk::Box` + Buttons |

## Label

Displays text with Pango rendering and SF Pro font.

```rust
let label = Label::new("Hello World")
    .font_size(24.0)
    .bold()
    .color(Color::WHITE)
    .max_width(200.0);

let view = View::new(label).with_frame(16.0, 16.0, 200.0, 30.0);
```

| Method | Description |
|---|---|
| `font_size(f32)` | Set font size |
| `font_family(impl Into<String>)` | Set font family |
| `bold()` | Set bold weight |
| `color(Color)` | Set text color |
| `max_width(f32)` | Set max width with ellipsize |

## Button

Clickable button with background color and corner radius.

```rust
let btn = Button::new("Click Me")
    .background(Color::from_hex("#FF6B2B").unwrap())
    .text_color(Color::WHITE)
    .corner_radius(8.0)
    .padding(16.0, 8.0)
    .on_click(|| println!("Clicked!"));

let view = View::new(btn).with_frame(16.0, 16.0, 120.0, 44.0);
```

| Method | Description |
|---|---|
| `background(Color)` | Set background color |
| `text_color(Color)` | Set text color |
| `corner_radius(f32)` | Set border radius |
| `padding(h, v)` | Set horizontal/vertical padding |
| `on_click(handler)` | Set click handler |
| `on_custom(action)` | Dispatch custom action to AppDelegate |

## TextField

Single-line text input with placeholder.

```rust
let field = TextField::new("Search...")
    .text("initial value")
    .password()
    .on_change(|text| println!("Changed: {}", text))
    .on_submit(|text| println!("Submitted: {}", text));

let view = View::new(field).with_frame(16.0, 16.0, 300.0, 44.0);
```

## ImageView

Displays an image from a file path.

```rust
let img = ImageView::new("/path/to/image.png")
    .size(128.0, 96.0)
    .opacity(0.8);

let view = View::new(img).with_frame(16.0, 16.0, 128.0, 96.0);
```

## Toggle

On/off switch with customizable colors.

```rust
let toggle = Toggle::new()
    .is_on(true)
    .on_color(Color::from_rgb(52, 199, 89))
    .off_color(Color::from_rgb(120, 120, 128))
    .on_change(|is_on| println!("Toggle: {}", is_on));

let view = View::new(toggle).with_frame(16.0, 16.0, 51.0, 31.0);
```

## Separator

Horizontal or vertical divider line.

```rust
let sep = Separator::horizontal()
    .color(Color::from_hex("#3a3a3d").unwrap())
    .thickness(1.0);

let view = View::new(sep).with_frame(16.0, 16.0, 400.0, 1.0);
```

## Icon

Icon display with optional tinting.

```rust
let icon = Icon::new("/usr/share/icons/setting.png")
    .size(24.0, 24.0)
    .tint(Color::WHITE);

let view = View::new(icon).with_frame(16.0, 16.0, 24.0, 24.0);
```

## ScrollView

Scrollable container for content.

```rust
let scroll = ScrollView::new()
    .content(inner_view)
    .vertical(true)
    .horizontal(false);

let view = View::new(scroll).with_frame(0.0, 0.0, 400.0, 600.0);
```

## ListView

Scrollable list of items with dividers.

```rust
let list = ListView::new()
    .items(vec!["Item 1", "Item 2", "Item 3"])
    .item_height(44.0)
    .show_dividers(true)
    .on_select(|index| println!("Selected: {}", index));

let view = View::new(list).with_frame(0.0, 0.0, 400.0, 600.0);
```

## Smooth Scrolling

Every UIKit scroll container (`ScrollView`, `ListView`, the `App` scroll
wrapper) uses smooth scrolling via `uikit::smooth_scroll`:

- Discrete mouse-wheel ticks animate to their target with an ease-out-cubic
  frame animation (`SCROLL_DURATION`, 200 ms) instead of jumping one step
  instantly. Rapid ticks retarget the running animation, so fast spins travel
  further but stay fluid.
- Touchpad (smooth/pixel) deltas apply 1:1 to preserve the native smooth
  feel; the animation target follows along so mixed input never fights.
- Kinetic touch scrolling and overlay scrollbars are enabled on every
  container.

```rust
let scrolled = gtk::ScrolledWindow::new();
uikit::smooth_scroll::apply_smooth_scrolling(&scrolled);
```

| Item | Description |
|---|---|
| `apply_smooth_scrolling(scrolled)` | Enable smooth scrolling on any `gtk::ScrolledWindow` |
| `WHEEL_STEP_PX` | Pixels per wheel tick (`64.0`, bounded by `MIN_STEP_PX` / `MAX_STEP_PX`) |
| `SCROLL_DURATION` | Settle animation length (`200 ms`) |
| `clamp_target(value, lower, upper, page_size)` | Clamp a target into the scrollable range |
| `ease_out_cubic(t)` | Easing curve used by the animation |
| `animated_value(from, to, elapsed)` | Interpolated value for the tick callback |
| `wheel_step(step_increment)` | Pixel travel for one tick from an adjustment step |

## TrafficLights

macOS-style window controls (close, minimize, maximize). Automatically added by `App` as window chrome.

```rust
let bar = TrafficLights::new()
    .without_maximize() // only close + minimize
    .size(17.0)
    .spacing(10.0);
```

| Method | Description |
|---|---|
| `at(x, y)` | Set position offsets |
| `size(f32)` | Set button diameter |
| `spacing(f32)` | Set gap between buttons |
| `show_maximize(bool)` | Show or hide the green maximize button (default `true`) |
| `without_maximize()` | Hide the green button, keep close and minimize |
| `minimize_enabled(bool)` | Enable the middle minimize button (default `true`); disabled stays gray and ignores clicks |
| `with_title(text)` | Centered title text (empty = no title) |
| `show_title(bool)` | Show or hide the title text (default `true`) |
| `without_title()` | Hide the title text |
| `with_custom(widget)` | Custom widget filling the bar after the reserved traffic lights; replaces the title zone |

The traffic lights always stay reserved on the left. A custom widget fills
the bar from after the lights to the right edge and lays out its own
alignment; the title only renders when no custom widget is set.

Apps use the bar without building it manually:

```rust
app.set_titlebar_widget(
    HStack::new()
        .spacing(8.0)
        .child(Button::new("Refresh").on_click(|| println!("refresh")))
        .child(Button::new("Share").on_click(|| println!("share"))),
);
app.show_titlebar_title(false); // optional, default `true`
```

| Method | Description |
|---|---|
| `App::set_titlebar_widget(widget)` | Custom title-bar content, rendered on every rebuild including the fullscreen reveal bar |
| `App::show_titlebar_title(bool)` | Show or hide the title-bar title text (default `true`) |

## Creating Custom Widgets

Implement `ViewContent` for custom rendering:

```rust
struct CardView {
    title: String,
    items: Vec<String>,
}

impl ViewContent for CardView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let card = gtk::Box::new(gtk::Orientation::Vertical, 8);
        // Build your custom GTK4 widget tree here
        card.upcast()
    }

    fn size_that_fits(&self, available: Size) -> Size {
        Size::new(available.width, 200.0)
    }
}
```

## Cross References

- [View.md](View.md) -- View base class and ViewContent trait
- [Shader.md](Shader.md) -- ShaderView for GPU rendering
- [MAIN.md](MAIN.md) -- overview
