use std::{borrow::Cow, fmt::Debug};

use super::Editor;
use crate::output::Output;

#[derive(Clone)]
pub struct Command {
    name: Cow<'static, str>,
    fun: fn(&mut Editor, &mut Output),
}

impl Command {
    pub fn new(name: impl Into<Cow<'static, str>>, fun: fn(&mut Editor, &mut Output)) -> Self {
        Self {
            name: name.into(),
            fun,
        }
    }

    pub fn call(&self, editor: &mut Editor, output: &mut Output) {
        (self.fun)(editor, output)
    }
}

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command").field("name", &self.name).finish()
    }
}

// Commands
pub fn dummy(editor: &mut Editor, _: &mut Output) {
    println!("test fn called");
}
