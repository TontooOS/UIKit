//! Event system for TontooUIKit.
//!
//! Events are mapped from GTK4 signals to UIKit event types.

use crate::style::Point;

/// Events that widgets can handle.
#[derive(Debug, Clone)]
pub enum Event {
    Click(ClickEvent),
    Hover(HoverEvent),
    HoverEnd(HoverEndEvent),
    Key(KeyEvent),
}

#[derive(Debug, Clone)]
pub struct ClickEvent {
    pub position: Point,
    pub button: MouseButton,
}

#[derive(Debug, Clone)]
pub struct HoverEvent {
    pub position: Point,
}

#[derive(Debug, Clone)]
pub struct HoverEndEvent;

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: Key,
    pub pressed: bool,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u32),
}

impl MouseButton {
    pub fn from_code(code: u32) -> Self {
        match code {
            0x110 => MouseButton::Left,
            0x112 => MouseButton::Middle,
            0x111 => MouseButton::Right,
            other => MouseButton::Other(other),
        }
    }
}

/// Named key codes for keyboard events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Escape, Return, Tab, Space, Backspace, Delete,
    Up, Down, Left, Right,
    Home, End, PageUp, PageDown, Insert,
    Other(u32),
}

impl Key {
    pub fn from_evdev(code: u32) -> Self {
        match code {
            // Letters (KEY_A=30 .. KEY_Z=44 contiguous-ish)
            30 => Key::A, 48 => Key::B, 46 => Key::C, 32 => Key::D,
            18 => Key::E, 33 => Key::F, 34 => Key::G, 35 => Key::H,
            23 => Key::I, 36 => Key::J, 37 => Key::K, 38 => Key::L,
            50 => Key::M, 49 => Key::N, 24 => Key::O, 25 => Key::P,
            16 => Key::Q, 19 => Key::R, 31 => Key::S, 20 => Key::T,
            22 => Key::U, 47 => Key::V, 44 => Key::W, 29 => Key::X,
            21 => Key::Y, 45 => Key::Z,
            // Number row (KEY_1=2 .. KEY_0=11)
            2 => Key::Num1, 3 => Key::Num2, 4 => Key::Num3,
            5 => Key::Num4, 6 => Key::Num5, 7 => Key::Num6,
            8 => Key::Num7, 9 => Key::Num8, 10 => Key::Num9,
            11 => Key::Num0,
            // Function keys (KEY_F1=59 .. KEY_F10=68, KEY_F11=87, KEY_F12=88)
            59 => Key::F1, 60 => Key::F2, 61 => Key::F3, 62 => Key::F4,
            63 => Key::F5, 64 => Key::F6, 65 => Key::F7, 66 => Key::F8,
            67 => Key::F9, 68 => Key::F10, 87 => Key::F11, 88 => Key::F12,
            // Special keys
            1 => Key::Escape, 28 => Key::Return, 15 => Key::Tab,
            57 => Key::Space, 14 => Key::Backspace, 111 => Key::Delete,
            // Navigation
            103 => Key::Up, 108 => Key::Down, 105 => Key::Left, 106 => Key::Right,
            102 => Key::Home, 107 => Key::End, 104 => Key::PageUp, 109 => Key::PageDown,
            110 => Key::Insert,
            other => Key::Other(other),
        }
    }

    pub fn to_char(&self, shift: bool) -> Option<char> {
        let base = match self {
            Key::A => 'a', Key::B => 'b', Key::C => 'c', Key::D => 'd',
            Key::E => 'e', Key::F => 'f', Key::G => 'g', Key::H => 'h',
            Key::I => 'i', Key::J => 'j', Key::K => 'k', Key::L => 'l',
            Key::M => 'm', Key::N => 'n', Key::O => 'o', Key::P => 'p',
            Key::Q => 'q', Key::R => 'r', Key::S => 's', Key::T => 't',
            Key::U => 'u', Key::V => 'v', Key::W => 'w', Key::X => 'x',
            Key::Y => 'y', Key::Z => 'z',
            Key::Num0 => '0', Key::Num1 => '1', Key::Num2 => '2',
            Key::Num3 => '3', Key::Num4 => '4', Key::Num5 => '5',
            Key::Num6 => '6', Key::Num7 => '7', Key::Num8 => '8',
            Key::Num9 => '9',
            Key::Space => ' ',
            _ => return None,
        };
        if shift {
            Some(base.to_uppercase().next().unwrap_or(base))
        } else {
            Some(base)
        }
    }
}

/// Side effects returned by event handlers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    None,
    Redraw,
    Focus(usize),
    Unfocus,
    Close,
    Minimize,
    Maximize,
    Resize { width: i32, height: i32 },
    SetTitle(String),
    Custom(String),
}

pub type EventHandler = Box<dyn Fn(&Event) -> Action + Send + Sync>;

pub fn noop_handler() -> EventHandler {
    Box::new(|_| Action::None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mouse_button_from_code() {
        assert_eq!(MouseButton::from_code(0x110), MouseButton::Left);
        assert_eq!(MouseButton::from_code(0x111), MouseButton::Right);
        assert_eq!(MouseButton::from_code(0x112), MouseButton::Middle);
        assert_eq!(MouseButton::from_code(999), MouseButton::Other(999));
    }

    #[test]
    fn action_equality() {
        assert_eq!(Action::None, Action::None);
        assert_eq!(Action::Redraw, Action::Redraw);
        assert_eq!(Action::Close, Action::Close);
        assert_ne!(Action::None, Action::Redraw);
    }
}
