use std::fmt::{Debug, Display};

use kaka_core::selection::Selection;

use crate::client::style::CursorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeKind {
    Normal,
    Insert,
    Visual,
}

impl ModeKind {
    pub const fn is_insert(&self) -> bool {
        matches!(self, Self::Insert)
    }

    pub const fn name(&self) -> &str {
        match self {
            Self::Insert => "insert",
            Self::Normal => "normal",
            Self::Visual => "visual",
        }
    }

    pub const fn cursor_kind(&self) -> CursorKind {
        match self {
            Self::Insert => CursorKind::Line,
            _ => CursorKind::Block,
        }
    }
}

impl Display for ModeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ModeData {
    Normal,
    Insert,
    Visual(Selection),
}

impl ModeData {
    pub const fn new(kind: ModeKind, pos: usize) -> Self {
        match kind {
            ModeKind::Normal => Self::Normal,
            ModeKind::Insert => Self::Insert,
            ModeKind::Visual => Self::Visual(Selection::at_pos(pos)),
        }
    }

    pub const fn kind(&self) -> ModeKind {
        match self {
            Self::Normal => ModeKind::Normal,
            Self::Insert => ModeKind::Insert,
            Self::Visual(_) => ModeKind::Visual,
        }
    }

    pub fn update(&mut self, pos: usize) {
        if let Self::Visual(selection) = self {
            selection.update_head(pos);
        }
    }
}
