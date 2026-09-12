# Cursor

Every UIKit window uses the TontooOS MacTahoe cursor pack at a fixed size of
`24` px, so the pointer keeps the same size and color while hovering or
resizing an app. The theme follows the app color scheme and stays in sync
with the compositor (`MacTahoe-dark-cursors` / `MacTahoe-cursors`) and BaseOS
(`XCURSOR_THEME` / `XCURSOR_SIZE`).

## Theme Tokens

| Constant | Type | Description |
|---|---|---|
| `CURSOR_SIZE` | `i32` | Cursor size in px, always `24` |
| `cursor_theme_for_scheme` | `fn` | Theme name for a color scheme (see below) |

### `cursor_theme_for_scheme`

```rust
pub fn cursor_theme_for_scheme(scheme: ColorScheme) -> &'static str
```

Return the cursor theme name for the scheme.

- Returns `"MacTahoe-dark-cursors"` when `scheme` is `ColorScheme::Dark`
- Returns `"MacTahoe-cursors"` when `scheme` is `ColorScheme::Light`

```rust
use uikit::app::{ColorScheme, cursor_theme_for_scheme};

let theme = cursor_theme_for_scheme(ColorScheme::Dark);
```

### `apply_cursor_theme`

```rust
pub fn apply_cursor_theme(scheme: ColorScheme)
```

Point GTK and the Xcursor environment at the scheme theme: sets
`gtk-cursor-theme-name` / `gtk-cursor-theme-size` on `gtk::Settings` when GTK
is initialized, and defaults `XCURSOR_THEME` / `XCURSOR_SIZE` when they are
missing or still point at a MacTahoe variant. Foreign user overrides are left
untouched. Safe to call before `gtk::init` (only the environment part runs)
and after (both parts run). Returns nothing.

- Sets the GTK cursor theme to the scheme pack at `24` px when settings exist
- Sets `XCURSOR_THEME` / `XCURSOR_SIZE` when unset or MacTahoe-based
- Returns nothing

```rust
use uikit::app::{ColorScheme, apply_cursor_theme};

apply_cursor_theme(ColorScheme::Dark);
```

## Rules

`App::run` applies the cursor theme automatically, so plain UIKit apps and
TontooUI apps (which render through UIKit `App`) need no manual setup:

- At startup `App::run` applies the resolved color scheme cursor theme
  (after system-theme detection, before the event loop starts).
- On every window activation the GTK settings are applied again, so each
  named cursor (`default`, `text`, `n-resize`, `se-resize`, ...) resolves
  from the same pack at the same size instead of falling back to Adwaita.
- The live system-theme watcher re-applies the cursor theme together with
  the CSS when the scheme flips between dark and light.

The eight invisible resize handles (`Window Resizing` in [MAIN.md](MAIN.md))
keep showing their resize cursors on the window edges and corners. The fix
does not remove that behavior; the resize arrows now come from the same
MacTahoe pack at `24` px, so hovering an edge no longer changes size or
color, only shape.

> **Note:** Speed-based cursor magnification (up to `2.5x` on fast movement)
> is owned by the compositor (`compositor/src/cursor.rs`), not by UIKit, and
> is intentionally left unchanged.

## Usage / Example

```rust
use uikit::prelude::*;

fn main() {
  let mut app = App::new("My App", 800, 600);
  app.set_color_scheme(ColorScheme::Dark);
  app.run();
}
```

No cursor code is needed. For custom event loops without `App::run`, call
`apply_cursor_theme` once after startup:

```rust
use uikit::app::{ColorScheme, apply_cursor_theme, mark_toolkit};

mark_toolkit();
apply_cursor_theme(ColorScheme::detect_system());
```

## Cross References

- [MAIN.md](MAIN.md) -- window resizing and resize-handle behavior
- [Style.md](Style.md) -- design tokens and color schemes
