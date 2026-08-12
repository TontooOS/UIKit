//! Image widget — displays an image using GtkPicture.

use crate::style::{Color, Padding};
use crate::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self, Picture};

pub struct Image {
    id: WidgetId,
    path: String,
    width: Option<f32>,
    height: Option<f32>,
    opacity: f32,
    position_mode: PositionMode,
    position: Position,
}

impl Image {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            id: next_widget_id(),
            path: path.into(),
            width: None,
            height: None,
            opacity: 1.0,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.position_mode = PositionMode::Absolute;
        self.position.x = Some(x);
        self.position.y = Some(y);
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self.position.width = Some(width);
        self.position.height = Some(height);
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self.position.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self.position.height = Some(height);
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn tint(self, _color: Color) -> Self {
        // GTK4 doesn't support direct tint; would need CSS filter.
        self
    }

    pub fn semi_transparent(mut self) -> Self {
        self.opacity = 0.5;
        self
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Widget for Image {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn position_mode(&self) -> PositionMode {
        self.position_mode
    }

    fn position(&self) -> Position {
        self.position
    }

    fn to_gtk(&self) -> gtk::Widget {
        let file = gtk::gio::File::for_path(&self.path);
        let picture = Picture::for_file(&file);

        if let Some(w) = self.width {
            picture.set_width_request(w as i32);
        }
        if let Some(h) = self.height {
            picture.set_height_request(h as i32);
        }

        picture.set_opacity(self.opacity as f64);
        picture.set_content_fit(gtk::ContentFit::Contain);

        picture.upcast()
    }

    fn is_interactive(&self) -> bool {
        false
    }

    fn padding(&self) -> Padding {
        Padding::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_builder() {
        let img = Image::new("/icons/app.png").size(128.0, 96.0).opacity(0.8);
        assert_eq!(img.path(), "/icons/app.png");
        assert_eq!(img.width, Some(128.0));
        assert_eq!(img.height, Some(96.0));
        assert_eq!(img.opacity, 0.8);
    }
}
