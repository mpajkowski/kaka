//! NOTE - copied from tui-rs
//!
//! `style` contains the primitives used to control how your user interface will look.

use bitflags::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    Rgb(u8, u8, u8),
    Indexed(u8),
}

bitflags! {
    /// Modifier changes the way a piece of text is displayed.
    ///
    /// They are bitflags so they can easily be composed.
    pub struct Modifier: u16 {
        const BOLD              = 0b0000_0000_0001;
        const DIM               = 0b0000_0000_0010;
        const ITALIC            = 0b0000_0000_0100;
        const UNDERLINED        = 0b0000_0000_1000;
        const SLOW_BLINK        = 0b0000_0001_0000;
        const RAPID_BLINK       = 0b0000_0010_0000;
        const REVERSED          = 0b0000_0100_0000;
        const HIDDEN            = 0b0000_1000_0000;
        const CROSSED_OUT       = 0b0001_0000_0000;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub add_modifier: Modifier,
    pub sub_modifier: Modifier,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fg: None,
            bg: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
    }
}

impl Style {
    /// Returns a `Style` resetting all properties.
    pub const fn reset() -> Self {
        Self {
            fg: Some(Color::Reset),
            bg: Some(Color::Reset),
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::all(),
        }
    }

    /// Changes the foreground color.
    pub const fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    /// Changes the background color.
    pub const fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    /// Changes the text emphasis.
    ///
    /// When applied, it adds the given modifier to the `Style` modifiers.
    pub fn add_modifier(mut self, modifier: Modifier) -> Self {
        self.sub_modifier.remove(modifier);
        self.add_modifier.insert(modifier);
        self
    }

    /// Changes the text emphasis.
    ///
    /// When applied, it removes the given modifier from the `Style` modifiers.
    pub fn remove_modifier(mut self, modifier: Modifier) -> Self {
        self.add_modifier.remove(modifier);
        self.sub_modifier.insert(modifier);
        self
    }

    /// Results in a combined style that is equivalent to applying the two individual styles to
    /// a style one after the other.
    pub fn patch(mut self, other: Self) -> Self {
        self.fg = other.fg.or(self.fg);
        self.bg = other.bg.or(self.bg);

        self.add_modifier.remove(other.sub_modifier);
        self.add_modifier.insert(other.add_modifier);
        self.sub_modifier.remove(other.add_modifier);
        self.sub_modifier.insert(other.sub_modifier);

        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CursorKind {
    Block,
    Line,
    Underscore,
}
