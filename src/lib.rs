//! # TontooUIKit
//!
//! A lightweight, native Rust UI toolkit for TontooOS.
//!
//! Uses GTK4 for rendering — Pango for text (subpixel AA, HarfBuzz).
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use tontoo_uikit::prelude::*;
//!
//! fn main() {
//!     let mut app = App::new("My App", 800, 600);
//!     app.set_root(
//!         VStack::new()
//!             .spacing(8.0)
//!             .child(Text::new("Hello, TontooOS!").font_size(24.0).bold())
//!             .child(Button::new("Click Me").on_click(|| println!("Clicked!"))),
//!     );
//!     app.run();
//! }
//! ```

pub mod widget;
pub mod layout;
pub mod widgets;
pub mod events;
pub mod style;
pub mod app;
pub mod animation;

pub const UITKIT_VERSION: (u32, u32, u32) = (0, 3, 0);

pub mod prelude {
    pub use crate::widget::{Widget, WidgetId, WidgetNode, PositionMode, Position};
    pub use crate::layout::{VStack, HStack, ZStack, PaddingWrap, Frame};
    pub use crate::widgets::{Text, Button, Image, TextField, TrafficLights};
    pub use crate::events::{Event, Action, Key, MouseButton};
    pub use crate::style::{Color, Font, FontWeight, Padding, Constraints, Size, Point};
    pub use crate::app::{App, AppDelegate, ColorScheme};
    pub use crate::UITKIT_VERSION;
}
