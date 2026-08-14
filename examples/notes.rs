//! # Simple Notes App
//!
//! Example showing a minimal notes app built with TontooUIKit.
//!
//! Run with: `cargo run --example notes`
//!
//! Features:
//! - Add a note via the text field (button or Enter)
//! - List all notes
//! - Delete a note
//!
//! State lives in the `NoteState` delegate; the text field content is kept
//! in a global buffer so it survives the view rebuild after each action.

use uikit::prelude::*;
use std::cell::RefCell;

/// Holds the current draft text of the input field across view rebuilds.
thread_local! {
    static DRAFT: RefCell<String> = RefCell::new(String::new());
}

struct NotesApp {
    notes: Vec<String>,
}

fn app_view(notes: &[String]) -> VStack {
    let mut stack = VStack::new().spacing(16.0);

    stack = stack.child(
        HStack::new()
            .spacing(8.0)
            .child(
                TextField::new("Type a note...")
                    .text(DRAFT.with(|d| d.borrow().clone()))
                    .on_change(|value| DRAFT.with(|d| *d.borrow_mut() = value))
                    .on_submit(|value| {
                        DRAFT.with(|d| *d.borrow_mut() = value);
                        uikit::app::dispatch_custom("notes_add");
                    }),
            )
            .child(Button::new("Add").on_custom("notes_add")),
    );

    if notes.is_empty() {
        stack = stack.child(
            Text::new("No notes yet — write your first one above.")
                .font_size(14.0)
                .color(Color::new(0.55, 0.55, 0.58, 1.0)),
        );
    }

    for (index, note) in notes.iter().enumerate() {
        let action = format!("notes_delete:{}", index);
        stack = stack.child(
            HStack::new()
                .spacing(8.0)
                .child(
                    Text::new(note)
                        .font_size(14.0)
                        .color(Color::new(0.92, 0.92, 0.94, 1.0))
                        .width(600.0),
                )
                .child(Button::new("Delete").background(Color::new(0.55, 0.16, 0.16, 1.0)).on_custom(action)),
        );
    }

    stack
}

impl AppDelegate for NotesApp {
    fn view(&self) -> Box<dyn Widget> {
        Box::new(
            PaddingWrap::new(Padding::all(28.0)).child(
                VStack::new()
                    .spacing(24.0)
                    .child(
                        Text::new("Notes")
                            .font_size(28.0)
                            .bold()
                            .color(Color::WHITE),
                    )
                    .child(app_view(&self.notes)),
            ),
        )
    }

    fn handle_custom(&mut self, action: &str) {
        match action {
            "notes_add" => {
                let text = DRAFT.with(|d| d.borrow().trim().to_string());
                if !text.is_empty() {
                    self.notes.push(text);
                    DRAFT.with(|d| d.borrow_mut().clear());
                }
            }
            other => {
                if let Some(index_str) = other.strip_prefix("notes_delete:") {
                    if let Ok(index) = index_str.parse::<usize>() {
                        if index < self.notes.len() {
                            self.notes.remove(index);
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let mut app = App::with_delegate("Notes", 800, 600, NotesApp { notes: Vec::new() });
    app.set_color_scheme(ColorScheme::Dark);
    app.run();
}
