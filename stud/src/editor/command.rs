use std::{borrow::Cow, fmt::Debug};

use super::Editor;
use crate::gui::Composer;

pub type CommandCallback = fn(&mut Editor, &mut Composer);

#[derive(Clone)]
pub struct Command {
    name: Cow<'static, str>,
    fun: CommandCallback,
}

impl Command {
    pub fn new(name: impl Into<Cow<'static, str>>, fun: CommandCallback) -> Self {
        Self {
            name: name.into(),
            fun,
        }
    }

    pub fn call(&self, editor: &mut Editor, composer: &mut Composer) {
        (self.fun)(editor, composer)
    }
}

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command").field("name", &self.name).finish()
    }
}

// Commands
pub fn dummy(_: &mut Editor, c: &mut Composer) {}
