use crossterm::event::KeyEvent;

use std::fmt::Debug;

#[derive(Debug)]
pub enum Mode {
    Xd,
    Insert,
    #[allow(unused)]
    Custom(Box<dyn CustomModeType>),
}

impl Mode {
    #[allow(unused)]
    pub fn custom<M: CustomModeType + 'static>(mode: M) -> Self {
        Self::Custom(Box::new(mode))
    }

    pub const fn is_insert(&self) -> bool {
        matches!(self, Self::Insert)
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Xd => "xd",
            Self::Insert => "insert",
            Self::Custom(c) => c.name(),
        }
    }
}

pub trait CustomModeType: Debug {
    fn name(&self) -> &str;
    fn on_key(&self, key_event: KeyEvent);
}
