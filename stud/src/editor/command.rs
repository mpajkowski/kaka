use std::{borrow::Cow, fmt::Debug};

use stud_core::ropey::Rope;

use super::Editor;

pub type CommandCallback = fn(&mut Editor);

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

    pub fn call(&self, editor: &mut Editor) {
        (self.fun)(editor)
    }
}

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command").field("name", &self.name).finish()
    }
}

// Commands
pub fn dummy(e: &mut Editor) {
    let (_, doc) = e.current_buffer_and_doc();
    doc.text_mut().append("a".into())
}

pub fn close(editor: &mut Editor) {
    editor.exit_code = Some(0);
}

#[macro_export]
macro_rules! command {
    ($fun: ident) => {{
        let name = stringify!($fun);
        Command::new(name, $fun)
    }};
}
