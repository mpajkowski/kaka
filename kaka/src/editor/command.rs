use std::{borrow::Cow, fmt::Debug};

use crate::current_mut;

use super::Editor;

pub type CommandFn = fn(&mut CommandData);

pub struct CommandData<'a> {
    pub editor: &'a mut Editor,
}

impl<'a> CommandData<'a> {
    pub fn new(editor: &'a mut Editor) -> Self {
        Self { editor }
    }
}

#[derive(Clone)]
pub struct Command {
    name: Cow<'static, str>,
    fun: CommandFn,
}

impl Command {
    pub fn new(name: impl Into<Cow<'static, str>>, fun: CommandFn) -> Self {
        Self {
            name: name.into(),
            fun,
        }
    }

    pub fn call(&self, context: &mut CommandData) {
        (self.fun)(context);
    }
}

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command").field("name", &self.name).finish()
    }
}

// commands impl
pub fn print_a(ctx: &mut CommandData) {
    let (_, doc) = current_mut!(ctx.editor);
    doc.text_mut().append("a".into());
}

pub fn close(ctx: &mut CommandData) {
    ctx.editor.exit_code = Some(0);
}

pub fn enter_insert_mode(ctx: &mut CommandData) {
    enter_mode_impl(ctx, "insert");
}

pub fn enter_xd_mode(ctx: &mut CommandData) {
    enter_mode_impl(ctx, "xd");
}

pub fn enter_normal_mode(ctx: &mut CommandData) {
    enter_mode_impl(ctx, "normal");
}

pub fn move_left(ctx: &mut CommandData) {
    let (buf, _) = current_mut!(ctx.editor);

    buf.current_char_in_line = buf.current_char_in_line.saturating_sub(1);
}

pub fn move_right(ctx: &mut CommandData) {
    let (buf, doc) = current_mut!(ctx.editor);

    let max_x = doc.text().line(buf.current_line).len_chars();

    if buf.current_char_in_line < max_x - 1 {
        buf.current_char_in_line += 1;
    }
}

pub fn move_up(ctx: &mut CommandData) {
    let (buf, doc) = current_mut!(ctx.editor);

    buf.current_line = buf.current_line.saturating_sub(1);
    let max_x = doc.text().line(buf.current_line).len_chars() - 1;
    buf.current_char_in_line = buf.current_char_in_line.min(max_x);
}

pub fn move_down(ctx: &mut CommandData) {
    let (buf, doc) = current_mut!(ctx.editor);

    let max_y = doc.text().len_lines();

    if buf.current_line < max_y - 1 {
        buf.current_line += 1;
    }

    let max_x = doc
        .text()
        .line(buf.current_line)
        .len_chars()
        .saturating_sub(1);

    buf.current_char_in_line = buf.current_char_in_line.min(max_x);
}

// impl

fn enter_mode_impl(ctx: &mut CommandData, mode: &str) {
    let (buf, _) = current_mut!(ctx.editor);
    buf.set_mode(mode);
}

#[macro_export]
macro_rules! command {
    ($fun: ident) => {{
        let name = stringify!($fun);
        Command::new(name, $fun)
    }};
}
