//! # Hello TontooOS - MINIMAL TEST
//!
//! Just a Text widget - to isolate if TrafficLights breaks the window.

use uikit::prelude::*;

fn main() {
    println!("TontooUIKit v{}.{}.{} (GTK4) MINIMAL", UITKIT_VERSION.0, UITKIT_VERSION.1, UITKIT_VERSION.2);

    let mut app = App::new("Hello TontooOS", 800, 600);
    app.set_root(
        VStack::new()
            .spacing(12.0)
            .child(Text::new("Hello, TontooOS!").font_size(28.0).bold().color(Color::WHITE)),
    );
    app.run();
}
