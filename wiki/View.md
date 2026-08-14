# View

`View` is the base class for all UI elements in TontooUIKit. Every visual element is either a `View` or wraps one. Custom views are created by implementing `ViewContent`.

## Creation

### `View::new(content)`

Create a view with custom content implementing `ViewContent`.

```rust
struct MyCard;

impl ViewContent for MyCard {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let card = gtk::Box::new(gtk::Orientation::Vertical, 8);
        card.upcast()
    }
}

let view = View::new(MyCard)
    .with_frame(16.0, 16.0, 300.0, 200.0)
    .with_background_color(Color::from_hex("#2a2a2c").unwrap());
```

### `View::empty()`

Create an empty container view (no content, used for grouping subviews).

```rust
let container = View::empty();
container.set_frame(0.0, 0.0, 400.0, 600.0);
container.add_subview(child_view);
```

### `View::from_gtk(widget)`

Wrap an existing GTK4 widget directly.

```rust
let label = gtk::Label::new(Some("Hello"));
let view = View::from_gtk(label.upcast());
```

## Frame and Bounds

| Method | Description |
|---|---|
| `set_frame(x, y, w, h)` | Set position and size |
| `with_frame(x, y, w, h)` | Builder variant |
| `frame()` / `bounds()` | Get frame or bounds rect |
| `center()` / `set_center(x, y)` | Get or set center point |
| `width()` / `height()` | Get dimensions |
| `set_width(w)` / `set_height(h)` | Set single dimension |

## Appearance

| Method | Description |
|---|---|
| `set_background_color(Some(color))` | Set background color |
| `set_hidden(bool)` | Show/hide view |
| `set_alpha(f32)` | Set opacity (0.0-1.0) |
| `set_tag(i32)` | Set numeric tag for lookup |

## View Hierarchy

| Method | Description |
|---|---|
| `add_subview(view)` | Add a child view |
| `remove_subview(id)` | Remove child by id |
| `remove_all_subviews()` | Remove all children |
| `view_by_id(id)` | Find view recursively |
| `view_by_tag(tag)` | Find view by tag |
| `bringSubviewToFront(id)` | Move to front |
| `sendSubviewToBack(id)` | Move to back |
| `subviews()` / `subviews_mut()` | Access children |

## Layout

| Method | Description |
|---|---|
| `set_needs_layout()` | Mark for relayout |
| `layout_if_needed()` | Force layout now |
| `layoutSubviews()` | Override in subclasses |

## Hit Testing

```rust
pub fn hit_test(&self, point: Point) -> Option<&View>
```

Determine which view is hit at a given point. Returns `None` if hidden or outside bounds. Checks subviews in reverse order (topmost first).

## Rendering

### `to_gtk()`

Standard rendering using `gtk::Overlay`. Sets `width_request`/`height_request` from frame. Used for non-scrollable views.

### `to_gtk_scrollable()`

Scrollable rendering using `gtk::Box` (vertical). Does not constrain height. Used inside `ScrolledWindow` containers.

```rust
let scrolled = gtk::ScrolledWindow::new();
let child = root.to_gtk_scrollable();
scrolled.set_child(Some(&child));
```

## ViewContent Trait

```rust
pub trait ViewContent {
    fn render(&self, frame: Rect) -> gtk::Widget;
    fn layout(&mut self, frame: Rect) {}
    fn did_move_to_superview(&mut self) {}
    fn will_move_from_superview(&mut self) {}
    fn did_appear(&mut self) {}
    fn will_disappear(&mut self) {}
    fn size_that_fits(&self, available: Size) -> Size { available }
    fn can_become_first_responder(&self) -> bool { false }
}
```

## ViewBuilder

Fluent builder for creating views:

```rust
let view = ViewBuilder::new(MyContent)
    .frame(0.0, 0.0, 200.0, 100.0)
    .background(Color::RED)
    .tag(42)
    .alpha(0.8)
    .subview(child_view)
    .build();
```

## Cross References

- [Widgets.md](Widgets.md) -- pre-built widgets implementing ViewContent
- [Shader.md](Shader.md) -- ShaderView for GPU rendering
- [Constraints.md](Constraints.md) -- layout constraints
- [MAIN.md](MAIN.md) -- overview
