# ViewController

View controllers manage a single screen of content, similar to Apple's `UIViewController`. They provide lifecycle methods and presentation navigation.

## ViewController

### `ViewController::new(delegate)`

Create a view controller with a delegate implementing `ViewControllerDelegate`.

```rust
struct SettingsScreen;

impl ViewControllerDelegate for SettingsScreen {
    fn load_view(&mut self) -> View {
        let mut root = View::empty();
        root.set_frame(0.0, 0.0, 800.0, 600.0);
        // Build your screen UI here
        root
    }

    fn view_did_load(&mut self) {
        println!("View loaded");
    }

    fn view_will_appear(&mut self) {
        println!("View will appear");
    }
}

let mut vc = ViewController::new(SettingsScreen)
    .with_title("Settings");
vc.load_view_if_needed();
```

### ViewControllerDelegate

```rust
pub trait ViewControllerDelegate {
    fn load_view(&mut self) -> View;
    fn view_did_load(&mut self) {}
    fn view_will_appear(&mut self) {}
    fn view_did_appear(&mut self) {}
    fn view_will_disappear(&mut self) {}
    fn view_did_disappear(&mut self) {}
    fn did_receive_memory_warning(&mut self) {}
}
```

### Methods

| Method | Description |
|---|---|
| `view()` / `view_mut()` | Get the view (loads if needed) |
| `load_view_if_needed()` | Load view if not loaded |
| `unload_view()` | Free view memory |
| `is_view_loaded()` | Check if loaded |
| `title()` / `set_title()` | Get/set title |
| `present(vc, animated)` | Present modally |
| `dismiss(animated)` | Dismiss presented VC |
| `push(vc, animated)` | Stub: no-op on `ViewController`; use `NavigationController::push` |
| `pop(animated)` | Stub: always returns `None`; use `NavigationController::pop` |
| `to_gtk()` | Render to GTK widget |

> **Note:** `ViewController::push` and `ViewController::pop` are stubs. Stack
> navigation is implemented on `NavigationController`.

### Lifecycle

```
load_view() -> view_did_load() -> view_will_appear() -> view_did_appear()
                                                           |
view_did_disappear() <- view_will_disappear() <---- dismiss/pop
```

## NavigationController

Manages a stack of view controllers with push/pop navigation.

```rust
let root_vc = ViewController::new(HomeScreen);
let mut nav = NavigationController::new(root_vc);

let detail_vc = ViewController::new(DetailScreen)
    .with_title("Detail");
nav.push(detail_vc, true);

// Later
nav.pop(true);
nav.pop_to_root(true);
```

### Methods

| Method | Description |
|---|---|
| `new(root)` | Create with root VC |
| `top()` / `top_mut()` | Get top VC |
| `root()` | Get root VC |
| `push(vc, animated)` | Push VC |
| `pop(animated)` | Pop top VC |
| `pop_to_root(animated)` | Pop to root |
| `depth()` | Get stack depth |

## App Integration

View controllers integrate with `App` via `AppDelegate`:

```rust
struct MyApp;

impl AppDelegate for MyApp {
    fn view(&self) -> Box<dyn Widget> {
        // Direct view (no VC)
        let mut root = View::empty();
        // ...
        Box::new(W(root))
    }
}
```

## Cross References

- [View.md](View.md) -- View base class
- [Constraints.md](Constraints.md) -- layout system
- [MAIN.md](MAIN.md) -- overview
