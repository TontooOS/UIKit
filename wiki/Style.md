# Style

`Style` holds shared design tokens for TontooUIKit: colors, fonts, spacing
types and window corner radii. Window radii match the MacTahoe GTK theme
`$wm_radius` value so UIKit windows and GTK windows look identical.

## Corner Radius Tokens

Window corners use two fixed values selected by `FormFactor`.

| Constant | Type | Description |
|---|---|---|
| `WINDOW_CORNER_RADIUS_DESKTOP` | `f32` | Window radius in desktop mode, `26.0` |
| `WINDOW_CORNER_RADIUS_LAPTOP` | `f32` | Window radius in laptop mode, `24.0` |

### `FormFactor`

Device form factor that selects the active window radius.

```rust
pub enum FormFactor {
  Desktop,
  Laptop,
}
```

### `FormFactor::window_corner_radius`

```rust
pub const fn window_corner_radius(self) -> f32
```

Return the window radius for the form factor (`26.0` for desktop,
`24.0` for laptop).

- Returns `26.0` when `self` is `Desktop`
- Returns `24.0` when `self` is `Laptop`

```rust
use uikit::style::FormFactor;

let radius = FormFactor::Desktop.window_corner_radius();
```

### `window_corner_radius_for`

```rust
pub const fn window_corner_radius_for(laptop: bool) -> f32
```

Return the window radius for a laptop flag (`false` is desktop `26.0`,
`true` is laptop `24.0`).

- Returns `26.0` when `laptop` is `false`
- Returns `24.0` when `laptop` is `true`

```rust
use uikit::style::window_corner_radius_for;

let desktop = window_corner_radius_for(false);
let laptop = window_corner_radius_for(true);
```

## App Integration

`App` stores a `FormFactor` (default `Desktop`) and always writes the
matching `border-radius` into the generated window CSS (`window`,
`window decoration`, content containers, `scrolledwindow` and `viewport`).
The window shadow uses the MacTahoe triple shadow so it follows the same
rounded shape, including a lighter `backdrop` variant. The live theme
watcher keeps the same radius and shadow when the color scheme changes.

## Window Edge

Every window gets a thin 1px edge plus an inner top highlight and an
outer ring, so the border stays visible against any wallpaper:

| Scheme | Edge (`border`) | Inner highlight (`inset`) | Outer ring |
|---|---|---|---|
| Dark | `rgba(255, 255, 255, 0.14)` (white) | `rgba(255, 255, 255, 0.08)` | `rgba(0, 0, 0, 0.75)` |
| Light | `rgba(0, 0, 0, 0.12)` (black) | `rgba(255, 255, 255, 0.9)` | `rgba(0, 0, 0, 0.12)` |

The edge is applied to both `window` and `window decoration` selectors,
combined with the drop shadow:

```css
window {
  border: 1px solid rgba(255, 255, 255, 0.14);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08), 0 3px 6px rgb(0 0 0 / 15%), 0 7px 24px rgb(0 0 0 / 12%), 0 12px 32px rgb(0 0 0 / 8%), 0 0 0 1px rgba(0, 0, 0, 0.75);
  margin: 24px;
}
```

The `24px` outer margin reserves space for the shadow on undecorated
(`decorated(false)`) windows so it is never clipped. Maximized, fullscreen
and tiled windows reset margin, radius and shadow to zero so they fill the
screen edge to edge.

## Window Frame

`App::no_window_frame()` disables the frame: no margin, no corner radius,
no border, no shadow (square content to the screen edge). The title bar
(if enabled) and the resize handles follow `no_window_bar` / `resizable`
as before. Default is framed. The live theme watcher keeps the frameless
override when the color scheme changes.

```rust
use uikit::prelude::*;

let mut app = App::new("My App", 800, 600);
app.no_window_frame();
```

| Method | Description |
|---|---|
| `no_window_frame()` | Disable margin, radius, border and shadow (default: framed) |

TontooUI apps use the same call: TontooUI has no own `App` type, the
UIKit `App` (re-exported through the TontooUI prelude) carries the API.

## Window Types

`App` supports two window chrome types via `WindowType` (no API change for
existing code — the default is unchanged):

| Type | Description |
|---|---|
| `Standard` (default) | Current UIKit window: slim 31px traffic-light bar, opaque background, no glass |
| `Mac` (opt-in) | macOS-style window: taller 44px decoration bar with centered title, built for the glass look |

```rust
use uikit::prelude::*;

let mut app = App::new("My App", 800, 600);
app.set_window_type(WindowType::Mac);
app.set_glass_strength(GlassStrength::BALANCED);
```

## Glass Strength

`GlassStrength` bundles transparency + blur with presets (default `OFF`
matches previous behavior — opaque, no blur). Free values stay available
via `App::set_window_transparency` / `App::set_window_blur`.

| Preset | Alpha | Blur |
|---|---|---|
| `OFF` (default) | `1.0` | `0.0` |
| `SUBTLE` | `0.92` | `12.0` |
| `BALANCED` | `0.85` | `20.0` |
| `STRONG` | `0.72` | `32.0` |

```rust
use uikit::prelude::*;

let mut app = App::new("My App", 800, 600);
app.set_glass_strength(GlassStrength::STRONG);
```

## Window Transparency

`App::set_window_transparency(alpha)` makes the window background
semi-transparent (`1.0` is the opaque default, clamped to `0.05..=1.0`).
The color stays adaptive per scheme:

| Scheme | Background |
|---|---|
| Dark | `rgba(30, 30, 30, alpha)` |
| Light | `rgba(245, 245, 247, alpha)` |

The alpha applies to `window`, `scrolledwindow` and `viewport` alike so
no opaque layer covers the see-through background. The live theme watcher
keeps the configured alpha when the color scheme changes.

## Title Bar

The traffic-light bar on top is always solid: the bar widget carries the
`.uikit-titlebar` class with the opaque scheme background (`#1E1E1E` in
dark mode, `#F5F5F7` in light mode), top rounded corners matching the
window radius and a 1px divider line at the bottom. The app title
(`App::new` title) is shown centered in the bar via
`TrafficLights::with_title` (`.uikit-titlebar-title`, 13px semibold);
lights stay on the left with a balancing spacer on the right.

Transparency and backdrop blur only affect the content below the bar: the
content sits directly on the single `window` background layer
(`scrolledwindow` and `viewport` stay transparent so no stacked darker box
appears).

```rust
use uikit::prelude::*;

let mut app = App::new("My App", 800, 600);
app.set_window_transparency(0.85);
```

## Window Backdrop Blur

`App::set_window_blur(sigma)` adds `backdrop-filter: blur()` to the window
background (`0.0` is off, clamped to `0.0..=100.0`). It needs a
semi-transparent background (`set_window_transparency` below `1.0`) to be
visible.

```rust
use uikit::prelude::*;

let mut app = App::new("My App", 800, 600);
app.set_window_transparency(0.85);
app.set_window_blur(20.0);
```

Support matrix (verified via web research, GTK 4.20+ NEWS and
`ext-background-effect` merge requests):

| Layer | Requirement | Effect |
|---|---|---|
| CSS property | GTK 4.20+ (older GTK ignores it) | `backdrop-filter: blur()` is parsed |
| In-window backdrop | GTK 4.20+ | Content behind the element inside the window is blurred |
| Behind-window blur | Compositor with `ext-background-effect-v1` (e.g. Hyprland 0.56+, Mutter support landing) | Wallpaper and windows behind the window are blurred |

Run the `glass_window` example to test:

```bash
UIKIT_DEMO_ALPHA=0.7 UIKIT_DEMO_BLUR=30 UIKIT_DEMO_SCHEME=light cargo run --example glass_window
```

| Method | Description |
|---|---|
| `set_form_factor(FormFactor)` | Set desktop or laptop radius mode |
| `set_laptop_mode(bool)` | Enable laptop mode (`true` is 24px, `false` is 26px) |
| `form_factor()` | Get the active form factor |
| `window_corner_radius()` | Get the active radius in px |

## Usage / Example

```rust
use uikit::prelude::*;

let mut app = App::new("My App", 800, 600);
app.set_laptop_mode(true);
assert_eq!(app.window_corner_radius(), 24.0);
```

Desktop mode is the default and needs no call:

```rust
use uikit::prelude::*;

let app = App::new("My App", 800, 600);
assert_eq!(app.window_corner_radius(), 26.0);
```

## Cross References

- [View.md](View.md) - base class for all UI elements
- [Widgets.md](Widgets.md) - pre-built widget catalog
- [MAIN.md](MAIN.md) - wiki entry point
