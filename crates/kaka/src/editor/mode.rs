use std::fmt::{Debug, Display};

use crate::client::style::CursorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Xd,
}

impl Mode {
    pub const fn is_insert(&self) -> bool {
        matches!(self, Self::Insert)
    }

    pub const fn name(&self) -> &str {
        match self {
            Self::Insert => "insert",
            Self::Normal => "normal",
            Self::Xd => "xd",
        }
    }

    pub const fn cursor_kind(&self) -> CursorKind {
        match self {
            Self::Normal => CursorKind::Block,
            Self::Insert => CursorKind::Line,
            Self::Xd => CursorKind::Block,
        }
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
