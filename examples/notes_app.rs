//! Example 2: Notes App with Custom Views
//!
//! Demonstrates a notes application with scrolling and multiple custom views.

use uikit::prelude::*;
use gtk::prelude::*;

struct NoteCardView {
    title: String,
    preview: String,
    timestamp: String,
    color: Color,
}

impl ViewContent for NoteCardView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let card = gtk::Box::new(gtk::Orientation::Vertical, 8);
        card.set_hexpand(true);
        card.set_vexpand(false);

        let accent_hex = format!(
            "#{:02x}{:02x}{:02x}",
            (self.color.r * 255.0) as u8,
            (self.color.g * 255.0) as u8,
            (self.color.b * 255.0) as u8,
        );

        let css = format!(
            "box {{ background-color: rgba(42, 42, 44, 0.9); border-radius: 12px; padding: 16px; border-left: 4px solid {accent_hex}; border: 1px solid rgba(255, 255, 255, 0.08); }}"
        );
        uikit::apply_css(&card, &css);

        let title_label = gtk::Label::new(Some(&self.title));
        title_label.set_halign(gtk::Align::Start);
        uikit::apply_css(&title_label, "label { font-family: 'SF Pro Display'; font-size: 15px; font-weight: bold; color: #ececec; }");
        card.append(&title_label);

        let preview_label = gtk::Label::new(Some(&self.preview));
        preview_label.set_halign(gtk::Align::Start);
        preview_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        preview_label.set_width_chars(60);
        uikit::apply_css(&preview_label, "label { font-family: 'SF Pro Display'; font-size: 12px; color: #8e8e93; }");
        card.append(&preview_label);

        let time_label = gtk::Label::new(Some(&self.timestamp));
        time_label.set_halign(gtk::Align::End);
        uikit::apply_css(&time_label, "label { font-family: 'SF Pro Display'; font-size: 11px; color: #636366; }");
        card.append(&time_label);

        if frame.width > 0.0 {
            card.set_width_request(frame.width as i32);
        }

        card.upcast()
    }
}

struct StatsBarView {
    stats: Vec<(String, String)>,
}

impl ViewContent for StatsBarView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.set_hexpand(true);
        container.set_height_request(60);
        uikit::apply_css(&container, "box { background-color: rgba(28, 28, 30, 0.9); border-radius: 12px; padding: 12px; }");

        let total = self.stats.len();
        for (i, (label, value)) in self.stats.iter().enumerate() {
            let stat_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
            stat_box.set_hexpand(true);
            stat_box.set_halign(gtk::Align::Center);

            let value_label = gtk::Label::new(Some(value));
            value_label.set_halign(gtk::Align::Center);
            uikit::apply_css(&value_label, "label { font-family: 'SF Pro Display'; font-size: 20px; font-weight: bold; color: #FF6B2B; }");
            stat_box.append(&value_label);

            let label_label = gtk::Label::new(Some(label));
            label_label.set_halign(gtk::Align::Center);
            uikit::apply_css(&label_label, "label { font-family: 'SF Pro Display'; font-size: 11px; color: #8e8e93; }");
            stat_box.append(&label_label);

            container.append(&stat_box);

            if i < total - 1 {
                let sep = gtk::Separator::new(gtk::Orientation::Vertical);
                uikit::apply_css(&sep, "separator { min-width: 1px; background-color: #3a3a3d; margin: 4px 8px; }");
                container.append(&sep);
            }
        }

        if frame.width > 0.0 {
            container.set_width_request(frame.width as i32);
        }

        container.upcast()
    }
}

struct NotesApp;

impl AppDelegate for NotesApp {
    fn view(&self) -> Box<dyn Widget> {
        let mut root = View::empty();
        root.set_frame(0.0, 0.0, 700.0, 900.0);

        let title = Label::new("Notes").font_size(28.0).bold().color(Color::WHITE);
        root.add_subview(View::new(title).with_frame(24.0, 0.0, 200.0, 36.0));

        let stats = StatsBarView {
            stats: vec![
                ("Total".into(), "12".into()),
                ("Today".into(), "3".into()),
                ("This Week".into(), "8".into()),
            ],
        };
        root.add_subview(View::new(stats).with_frame(24.0, 0.0, 652.0, 60.0));

        let notes = vec![
            NoteCardView { title: "Shopping List".into(), preview: "Milk, eggs, bread, butter, cheese, pasta, tomatoes, olive oil, garlic, onions, chicken breast, rice".into(), timestamp: "Today".into(), color: Color::from_rgb(255, 107, 43) },
            NoteCardView { title: "Meeting Notes - Sprint Review".into(), preview: "Discussed project timeline, milestones, and deliverables. Need to finish UIKit refactor by end of week. Also review the LiquidGlass integration.".into(), timestamp: "Today".into(), color: Color::from_rgb(0, 122, 255) },
            NoteCardView { title: "TontooOS Ideas".into(), preview: "Build a custom OS with liquid glass UI, spring physics animations, custom Wayland compositor, curated app store, and Raspberry Pi backend for sync.".into(), timestamp: "Yesterday".into(), color: Color::from_rgb(52, 199, 89) },
            NoteCardView { title: "Travel Plans - Berlin".into(), preview: "Flight: Aug 20, Hotel: Mitte district, Visit TontooOS HQ, Meet the team, Review roadmap for Q4".into(), timestamp: "Aug 12".into(), color: Color::from_rgb(175, 82, 222) },
            NoteCardView { title: "Reading List".into(), preview: "The Rust Programming Language, Linux Kernel Development, Designing Data-Intensive Applications, Architecture of Open Source Applications".into(), timestamp: "Aug 10".into(), color: Color::from_rgb(255, 204, 0) },
            NoteCardView { title: "Workout Routine".into(), preview: "Monday: Chest & Triceps, Tuesday: Back & Biceps, Wednesday: Rest, Thursday: Legs, Friday: Shoulders & Arms".into(), timestamp: "Aug 9".into(), color: Color::from_rgb(255, 69, 58) },
            NoteCardView { title: "Rust Tips".into(), preview: "Use Cow<str> for optional string allocation, Arc<Mutex<T>> for shared state, impl Trait for return position, enum dispatch for small dynamic dispatch".into(), timestamp: "Aug 8".into(), color: Color::from_rgb(0, 199, 190) },
            NoteCardView { title: "Grocery Budget".into(), preview: "Weekly budget: $80, Spent: $65, Remaining: $15, Categories: Produce $30, Dairy $15, Meat $20".into(), timestamp: "Aug 7".into(), color: Color::from_rgb(255, 149, 0) },
            NoteCardView { title: "Project Milestones".into(), preview: "Week 1: Base OS bootable, Week 2: UIKit framework, Week 3: Shell + Dock, Week 4: Apps (Finder, Terminal, Settings), Week 5: App Store".into(), timestamp: "Aug 6".into(), color: Color::from_rgb(0, 122, 255) },
            NoteCardView { title: "Birthday Gifts".into(), preview: "Mom: Book about gardening, Dad: New headphones, Sister: Art supplies, Brother: Rust programming book".into(), timestamp: "Aug 5".into(), color: Color::from_rgb(255, 45, 85) },
            NoteCardView { title: "API Design Notes".into(), preview: "REST endpoints: /api/v1/users, /api/v1/notes, /api/v1/sync. Auth via JWT tokens. Rate limiting: 100 req/min. WebSocket for real-time sync.".into(), timestamp: "Aug 4".into(), color: Color::from_rgb(48, 209, 88) },
            NoteCardView { title: "Movie Watchlist".into(), preview: "Dune: Part Two, Oppenheimer, The Batman, Everything Everywhere All at Once, Top Gun: Maverick, Spider-Man: Across the Spider-Verse".into(), timestamp: "Aug 3".into(), color: Color::from_rgb(191, 90, 242) },
        ];

        let mut y = 0.0;
        for note in notes {
            root.add_subview(View::new(note).with_frame(24.0, 0.0, 652.0, 110.0));
            y += 122.0;
        }

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
    let mut app = App::new("Notes App", 700, 900);
    app.set_color_scheme(ColorScheme::Dark);
    app.set_delegate(NotesApp);
    app.run();
}
