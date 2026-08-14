//! Example 1: Settings App with Custom Views
//!
//! Demonstrates scrolling, resizable window, and custom views.

use uikit::prelude::*;
use gtk::prelude::*;

struct CardView {
    title: String,
    items: Vec<String>,
}

impl ViewContent for CardView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let card = gtk::Box::new(gtk::Orientation::Vertical, 12);
        card.set_hexpand(true);
        card.set_vexpand(false);

        let css = format!(
            "box {{ background-color: rgba(42, 42, 44, 0.8); border-radius: 12px; padding: 16px; border: 1px solid rgba(255, 255, 255, 0.1); }}"
        );
        uikit::apply_css(&card, &css);

        let title_label = gtk::Label::new(Some(&self.title));
        title_label.set_halign(gtk::Align::Start);
        uikit::apply_css(&title_label, "label { font-family: 'SF Pro Display'; font-size: 16px; font-weight: bold; color: #ececec; }");
        card.append(&title_label);

        let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
        uikit::apply_css(&sep, "separator { min-height: 1px; background-color: #3a3a3d; margin: 4px 0; }");
        card.append(&sep);

        for item in &self.items {
            let lbl = gtk::Label::new(Some(item));
            lbl.set_halign(gtk::Align::Start);
            uikit::apply_css(&lbl, "label { font-family: 'SF Pro Display'; font-size: 13px; color: #ececec; padding: 4px 0; }");
            card.append(&lbl);
        }

        if frame.width > 0.0 {
            card.set_width_request(frame.width as i32);
        }

        card.upcast()
    }
}

struct ToggleRowView {
    label: String,
    is_on: bool,
}

impl ViewContent for ToggleRowView {
    fn render(&self, _frame: Rect) -> gtk::Widget {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        row.set_hexpand(true);
        row.set_height_request(44);

        let lbl = gtk::Label::new(Some(&self.label));
        lbl.set_halign(gtk::Align::Start);
        lbl.set_hexpand(true);
        uikit::apply_css(&lbl, "label { font-family: 'SF Pro Display'; font-size: 13px; color: #ececec; }");
        row.append(&lbl);

        let toggle = Toggle::new().is_on(self.is_on);
        let toggle_view = View::new(toggle);
        row.append(&toggle_view.to_gtk());

        row.upcast()
    }
}

struct SettingsApp;

impl AppDelegate for SettingsApp {
    fn view(&self) -> Box<dyn Widget> {
        let mut root = View::empty();
        root.set_frame(0.0, 0.0, 600.0, 900.0);

        // Title
        let title = Label::new("Settings").font_size(28.0).bold().color(Color::WHITE);
        root.add_subview(View::new(title).with_frame(24.0, 0.0, 200.0, 36.0));

        let mut y = 0.0;

        // Profile card
        let card1 = CardView {
            title: "Profile".into(),
            items: vec!["Name: Arlo".into(), "Email: arlo@tontoo.os".into(), "Bio: Building TontooOS".into()],
        };
        root.add_subview(View::new(card1).with_frame(24.0, 0.0, 552.0, 160.0));

        // Toggles
        for (label, on) in [("Dark Mode", true), ("Notifications", false), ("Auto-Update", true), ("Bluetooth", false)] {
            let toggle = ToggleRowView { label: label.into(), is_on: on };
            root.add_subview(View::new(toggle).with_frame(24.0, 0.0, 552.0, 44.0));
        }

        // Separator
        let sep = Separator::horizontal().color(Color::from_hex("#3a3a3d").unwrap());
        root.add_subview(View::new(sep).with_frame(24.0, 0.0, 552.0, 1.0));

        // System card
        let card2 = CardView {
            title: "System".into(),
            items: vec![
                "Version: 0.1.0".into(),
                "Build: 2026.08.13".into(),
                "Kernel: 6.18".into(),
                "Desktop: TontooCompositor".into(),
                "UI Framework: TontooUIKit".into(),
            ],
        };
        root.add_subview(View::new(card2).with_frame(24.0, 0.0, 552.0, 240.0));

        // About card
        let card3 = CardView {
            title: "About TontooOS".into(),
            items: vec![
                "A macOS-inspired Linux Desktop".into(),
                "Built with Rust + GTK4".into(),
                "LiquidGlass UI Framework".into(),
                "Designed for Daily Driver use".into(),
                "Custom Wayland Compositor".into(),
                "Curated App Store".into(),
                "SF Pro Typography".into(),
            ],
        };
        root.add_subview(View::new(card3).with_frame(24.0, 0.0, 552.0, 300.0));

        // Search at bottom
        let search = TextField::new("Search settings...");
        root.add_subview(View::new(search).with_frame(24.0, 0.0, 552.0, 44.0));

        // Wrap in scrollable container
        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_hscrollbar_policy(gtk::PolicyType::Never);
        scrolled.set_vscrollbar_policy(gtk::PolicyType::Automatic);
        scrolled.set_kinetic_scrolling(true);
        let child = root.to_gtk_scrollable();
        child.set_hexpand(true);
        child.set_vexpand(true);
        scrolled.set_child(Some(&child));

        struct W(gtk::ScrolledWindow);
        impl Widget for W {
            fn id(&self) -> uikit::widget::WidgetId { 0 }
            fn to_gtk(&self) -> gtk::Widget { self.0.clone().upcast() }
        }
        Box::new(W(scrolled))
    }
}

fn main() {
    let mut app = App::new("Settings App", 600, 900);
    app.set_color_scheme(ColorScheme::Dark);
    app.set_delegate(SettingsApp);
    app.run();
}
