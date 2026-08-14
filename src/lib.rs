//! # TontooUIKit
//!
//! A lightweight, native Rust UI toolkit for TontooOS.
//!
//! Uses GTK4 for rendering — Pango for text (subpixel AA, HarfBuzz).
//!
//! ## Architecture
//!
//! TontooUIKit follows Apple's UIKit design philosophy:
//!
//! - **[`View`]`** — the base class for all UI elements
//! - **[`ViewContent`]`** — trait for custom view rendering
//! - **Widgets** — pre-built elements (Label, Button, TextField, etc.)
//! - **[`ViewController`]`** — manages a single screen of content
//! - **[`LayoutConstraint`]`** — Auto Layout constraint system
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use uikit::prelude::*;
//!
//! fn main() {
//!     let mut app = App::new("My App", 800, 600);
//!     app.set_root(
//!         View::new(
//!             VStack::new()
//!                 .spacing(8.0)
//!                 .child(Text::new("Hello, TontooOS!").font_size(24.0).bold())
//!                 .child(Button::new("Click Me").on_click(|| println!("Clicked!"))),
//!         )
//!     );
//!     app.run();
//! }
//! ```
//!
//! ## Custom Views
//!
//! Create custom views by implementing [`ViewContent`]:
//!
//! ```rust,no_run
//! use uikit::prelude::*;
//!
//! struct MyCustomView;
//!
//! impl ViewContent for MyCustomView {
//!     fn render(&self, frame: Rect) -> gtk::Widget {
//!         let label = gtk::Label::new(Some("Custom View"));
//!         label.upcast()
//!     }
//! }
//!
//! let view = View::new(MyCustomView)
//!     .with_frame(0.0, 0.0, 200.0, 100.0);
//! ```

pub mod view;
pub mod constraints;
pub mod controller;
pub mod widget;
pub mod layout;
pub mod widgets;
pub mod events;
pub mod style;
pub mod app;
pub mod animation;
pub mod shader;
pub mod shader_view;

/// Re-export of the CSS helper for applying inline styles.
pub use widget::apply_css;

pub const UITKIT_VERSION: (u32, u32, u32) = (26, 1, 0);

pub mod prelude {
    // Core types
    pub use crate::view::{View, ViewContent, ViewId, ViewBuilder, LayoutConstraint, Anchor, ConstraintRelation};
    pub use crate::constraints::{
        ConstraintSolver, EdgeSet, constraint, constraint_with_constant, constraint_full,
        pin_to_superview, center_in_superview, set_size,
    };
    pub use crate::controller::{ViewController, ViewControllerDelegate, NavigationController};

    // Widget trait (for backward compatibility)
    pub use crate::widget::{Widget, WidgetId, WidgetNode, PositionMode, Position};

    // Layout (legacy - will be deprecated in favor of constraints)
    pub use crate::layout::{VStack, HStack, ZStack, PaddingWrap, Frame};

    // Widgets
    pub use crate::widgets::{
        Label, Text, Button, ImageView, Image, TextField, Toggle, Separator, SeparatorOrientation,
        Icon, ScrollView, ListView, TrafficLights,
    };

    // Events
    pub use crate::events::{Event, Action, Key, MouseButton};

    // Style
    pub use crate::style::{Color, Font, FontWeight, Padding, Constraints, Size, Point, Rect, HAlignment, VAlignment, Alignment};

    // App
    pub use crate::app::{App, AppDelegate, ColorScheme};

    // Shaders
    pub use crate::shader::{Shader, Uniforms, ShaderError, VERTEX_FULLSCREEN, render_fullscreen};
    pub use crate::shader_view::{
        ShaderView, gradient_view, blur_view, glass_view, noise_view, rainbow_view, wave_view,
    };

    // Version
    pub use crate::UITKIT_VERSION;
}
