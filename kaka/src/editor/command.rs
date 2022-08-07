use std::{borrow::Cow, fmt::Debug};

use crate::current_mut;

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
        (self.fun)(editor);
    }
}

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command").field("name", &self.name).finish()
    }
}

// Commands
pub fn print_a(e: &mut Editor) {
    let (_, doc) = current_mut!(e);
    doc.text_mut().append("a".into());
}

pub fn close(editor: &mut Editor) {
    editor.exit_code = Some(0);
}

pub fn enter_insert_mode(editor: &mut Editor) {
    enter_mode_impl(editor, "insert");
}

pub fn enter_xd_mode(editor: &mut Editor) {
    enter_mode_impl(editor, "xd");
}

fn enter_mode_impl(editor: &mut Editor, mode: &str) {
    let (buf, _) = current_mut!(editor);
    buf.set_mode(mode);
}

#[macro_export]
macro_rules! command {
    ($fun: ident) => {{
        let name = stringify!($fun);
        Command::new(name, $fun)
    }};
}
