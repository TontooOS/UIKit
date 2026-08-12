//! # Hello TontooOS
//!
//! Example app demonstrating TontooUIKit with GTK4 + TrafficLights.
//!
//! Run with: `cargo run --example hello`

use tontoo_uikit::prelude::*;

struct HelloDelegate {
    count: u32,
}

impl AppDelegate for HelloDelegate {
    fn view(&self) -> Box<dyn Widget> {
        Box::new(
            VStack::new()
                .spacing(12.0)
                .child(Text::new("Welcome to TontooOS").font_size(28.0).bold().color(Color::WHITE))
                .child(Text::new("Native GTK4 rendering").font_size(14.0).color(Color::new(0.70, 0.70, 0.72, 1.0)))
                .child(Button::new(format!("Clicked {} times", self.count)).on_custom("increment"))
                .child(
                    Button::new("Reset")
                        .background(Color::new(0.25, 0.25, 0.27, 1.0))
                        .text_color(Color::new(0.92, 0.92, 0.94, 1.0))
                        .on_custom("reset"),
                ),
        )
    }

    fn handle_custom(&mut self, action: &str) {
        match action {
            "increment" => self.count += 1,
            "reset" => self.count = 0,
            _ => {}
        }
    }
}

fn main() {
    println!(
        "TontooUIKit v{}.{}.{} (GTK4)",
        UITKIT_VERSION.0, UITKIT_VERSION.1, UITKIT_VERSION.2
    );

    let delegate = HelloDelegate { count: 0 };
    let mut app = App::with_delegate("Hello TontooOS", 800, 600, delegate);
    app.set_glass(0.25, 0.65, 20.0);
    app.set_color_scheme(ColorScheme::Dark);

    app.run();
}
