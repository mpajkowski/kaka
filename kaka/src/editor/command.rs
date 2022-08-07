use std::{borrow::Cow, fmt::Debug};

use crossterm::event::{KeyCode, KeyEvent};
use kaka_core::document::Document;

use crate::{current, current_mut};

use super::{Buffer, Editor};

pub type CommandFn = fn(&mut CommandData);

pub struct CommandData<'a> {
    pub editor: &'a mut Editor,
    pub trigger: KeyEvent,
}

impl<'a> CommandData<'a> {
    pub fn new(editor: &'a mut Editor, key_event: KeyEvent) -> Self {
        Self {
            editor,
            trigger: key_event,
        }
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

pub fn save(ctx: &mut CommandData) {
    let (_, doc) = current!(ctx.editor);

    doc.save().unwrap();
}

pub fn switch_to_insert_mode_before(ctx: &mut CommandData) {
    let (buf, _) = current_mut!(ctx.editor);

    buf.switch_mode("insert");
}

pub fn switch_to_insert_mode_after(ctx: &mut CommandData) {
    let (buf, doc) = current_mut!(ctx.editor);

    buf.switch_mode("insert");
    buf.text_position = (buf.text_position + 1).min(doc.text().len_chars() - 1);
}

pub fn switch_to_xd_mode(ctx: &mut CommandData) {
    let (buf, _) = current_mut!(ctx.editor);

    buf.switch_mode("xd");
}

pub fn switch_to_normal_mode(ctx: &mut CommandData) {
    let (buf, doc) = current_mut!(ctx.editor);

    let was_insert = buf.mode().is_insert();
    buf.switch_mode("normal");

    // move one cell left when exiting insert mode
    if was_insert {
        let text = doc.text();
        let line_idx = text.char_to_line(buf.text_position);
        let line_start = text.line_to_char(line_idx);

        buf.text_position = buf.text_position.saturating_sub(1).max(line_start);
    }
}

pub fn move_left(ctx: &mut CommandData) {
    let (buf, doc) = current_mut!(ctx.editor);

    let pos = buf.text_position;
    let text = doc.text();
    let line_idx = text.char_to_line(pos);

    let line_start_idx = text.line_to_char(line_idx);
    let current_x = pos - line_start_idx;

    if current_x > 0 {
        buf.text_position = buf.text_position.saturating_sub(1);
        buf.saved_column = current_x.saturating_sub(1);
    }
}

pub fn move_right(ctx: &mut CommandData) {
    let (buf, doc) = current_mut!(ctx.editor);

    let text = doc.text();
    let current_line_idx = text.char_to_line(buf.text_position);
    let current_line_start = text.line_to_char(current_line_idx);
    let next_line_start = text.line_to_char(current_line_idx + 1);

    let new_pos = buf.text_position + 1;

    let in_line = (current_line_start..next_line_start.saturating_sub(1)).contains(&new_pos);
    let last_line = current_line_idx == text.len_lines() - 1 && new_pos < text.len_chars();

    if in_line || last_line {
        buf.text_position = new_pos;
        buf.saved_column = new_pos - current_line_start;
    }
}

pub fn move_up(ctx: &mut CommandData) {
    let (buf, doc) = current_mut!(ctx.editor);

    let text = doc.text();
    let current_line_idx = text.char_to_line(buf.text_position);

    if current_line_idx > 0 {
        let prev_end = text.line_to_char(current_line_idx) - 1;
        let prev_start = text.line_to_char(current_line_idx - 1);

        buf.text_position = prev_start
            + (buf.saved_column).min(prev_end.saturating_sub(prev_start).saturating_sub(1));
    }
}

pub fn move_down(ctx: &mut CommandData) {
    let (buf, doc) = current_mut!(ctx.editor);

    let text = doc.text();

    let current_line_idx = text.char_to_line(buf.text_position);

    // can't go beyond the last line
    if current_line_idx == text.len_lines().saturating_sub(1) {
        return;
    }

    let next_start = text.line_to_char(current_line_idx + 1);
    let mut next_end = text.line_to_char(current_line_idx + 2);

    // handle special case: last line not ended with newline
    if text.chars_at(next_end).reversed().next() == Some('\n') {
        next_end -= 1;
    }

    buf.text_position =
        next_start + (buf.saved_column).min(next_end.saturating_sub(next_start).saturating_sub(1));
}

pub fn remove_char(ctx: &mut CommandData) {
    let (buf, doc) = current_mut!(ctx.editor);

    let text = doc.text_mut();

    let current_line_idx = text.char_to_line(buf.text_position);
    let current_line_start = text.line_to_char(current_line_idx);
    let current_line_end = text.line_to_char(current_line_idx + 1);

    if (current_line_start..current_line_end).contains(&buf.text_position) {
        let pos = buf.text_position;

        let _ = text.try_remove(pos..pos + 1);

        if pos == current_line_end.saturating_sub(2) {
            buf.text_position = (pos.saturating_sub(1)).max(current_line_start);
        }
    }
}

pub fn insert_mode_on_key(ctx: &mut CommandData, event: KeyEvent) {
    let (buf, doc) = current_mut!(ctx.editor);

    let text = doc.text_mut();
    let pos = buf.text_position;

    match event.code {
        KeyCode::Char(c) => {
            text.insert_char(pos, c);
            buf.text_position += 1;
        }
        KeyCode::Backspace => {
            let new_pos = pos.saturating_sub(1);
            text.remove(new_pos..pos);
            buf.text_position = new_pos;
        }
        KeyCode::Enter => {
            text.insert_char(pos, '\n');
            buf.text_position += 1;
        }
        KeyCode::Left => buf.text_position = buf.text_position.saturating_sub(1),
        KeyCode::Right => buf.text_position = (buf.text_position + 1).min(text.len_chars() - 1),
        _ => { /* TODO */ }
    };
}

pub fn buffer_next(ctx: &mut CommandData) {
    let curr = ctx.editor.current;

    let mut iter = ctx.editor.buffers.keys();

    let next = iter
        .clone()
        .find(|id| **id > curr)
        .or_else(|| iter.next())
        .unwrap();

    ctx.editor.current = *next;
}

pub fn buffer_prev(ctx: &mut CommandData) {
    let curr = ctx.editor.current;

    let mut iter = ctx.editor.buffers.keys().rev();

    let prev = iter
        .clone()
        .find(|id| **id < curr)
        .or_else(|| iter.next())
        .unwrap();

    ctx.editor.current = *prev;
}

pub fn buffer_create(ctx: &mut CommandData) {
    let scratch = Document::new_scratch();
    let buffer = Buffer::new_text(0, &scratch).unwrap();

    let doc_id = scratch.id();
    let buf_id = buffer.id();

    ctx.editor.documents.insert(doc_id, scratch);
    ctx.editor.buffers.insert(buf_id, buffer);
    ctx.editor.current = buf_id;
}

pub fn buffer_kill(ctx: &mut CommandData) {
    let immortal = ctx
        .editor
        .buffers
        .get(&ctx.editor.current)
        .unwrap()
        .immortal();

    if !immortal {
        ctx.editor.buffers.remove(&ctx.editor.current);

        if ctx.editor.buffers.is_empty() {
            buffer_create(ctx);
        } else {
            buffer_prev(ctx);
        }
    }
}

#[macro_export]
macro_rules! command {
    ($fun: ident) => {{
        let name = stringify!($fun);
        Command::new(name, $fun)
    }};
}

#[cfg(test)]
mod test {
    use crossterm::event::KeyModifiers;
    use kaka_core::{document::Document, ropey::Rope};

    use crate::editor::Buffer;

    use super::*;

    #[track_caller]
    fn test_cmd<B: FnOnce(&Buffer) -> bool>(
        start_position: usize,
        text: impl AsRef<str>,
        command: fn(&mut CommandData),
        checks_buffer: impl IntoIterator<Item = B>,
    ) {
        let mut editor = Editor::init();

        let mut document = Document::new_scratch();
        *document.text_mut() = Rope::from(text.as_ref());

        let buffer = Buffer::new_text(start_position, &document).unwrap();

        editor.add_buffer_and_document(buffer, document, true);

        let mut data = CommandData::new(
            &mut editor,
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
        );

        command(&mut data);

        let (buf, _) = current!(data.editor);

        for check in checks_buffer.into_iter() {
            assert!(check(buf), "Buffer assert failed: {buf:#?}");
        }
    }

    // to save characters :P
    type B<'a> = &'a Buffer;

    #[test]
    #[rustfmt::skip]
    fn move_left_prevented_on_pos_0() {
        test_cmd(0, "kakaka\n", move_left, [|buf: B| buf.text_position == 0, |buf: B| buf.saved_column == 0]);
        test_cmd(7, "kakaka\nkaka", move_left, [|buf: B| buf.text_position == 7]);
    }

    #[test]
    #[rustfmt::skip]
    fn move_left_doable_until_newline() {
        test_cmd(3, "kaka\n", move_left, [|buf: B| buf.text_position == 2, |buf: B| buf.saved_column == 2]);
        test_cmd(2, "kaka\n", move_left, [|buf: B| buf.text_position == 1, |buf: B| buf.saved_column == 1]);
        test_cmd(1, "kaka\n", move_left, [|buf: B| buf.text_position == 0, |buf: B| buf.saved_column == 0]);
    }

    #[test]
    #[rustfmt::skip]
    fn move_right_doable_until_newline() {
        test_cmd(0, "kaka\n", move_right, [|buf: B| buf.text_position == 1, |buf: B| buf.saved_column == 1]);
        test_cmd(1, "kaka\n", move_right, [|buf: B| buf.text_position == 2, |buf: B| buf.saved_column == 2]);
        test_cmd(2, "kaka\n", move_right, [|buf: B| buf.text_position == 3, |buf: B| buf.saved_column == 3]);
        test_cmd(3, "kaka\n", move_right, [|buf: B| buf.text_position == 3, |buf: B| buf.saved_column == 3]);
        test_cmd(5, "kaka\nkaka", move_right, [|buf: B| buf.text_position == 6, |buf: B| buf.saved_column == 1]);
        test_cmd(6, "kaka\nkaka", move_right, [|buf: B| buf.text_position == 7, |buf: B| buf.saved_column == 2]);
        test_cmd(7, "kaka\nkaka", move_right, [|buf: B| buf.text_position == 8, |buf: B| buf.saved_column == 3]);
        test_cmd(7, "kaka\nkaka\n", move_right, [|buf: B| buf.text_position == 8, |buf: B| buf.saved_column == 3]);
    }

    #[test]
    #[rustfmt::skip]
    fn move_down_simple() {
        let text = "012\n456\n890";
        test_cmd(0, text, move_down, [|buf: B| buf.text_position == 4]);
        test_cmd(4, text, move_down, [|buf: B| buf.text_position == 8]);
        test_cmd(8, text, move_down, [|buf: B| buf.text_position == 8]);

        test_cmd(1, text, move_down, [|buf: B| buf.text_position == 5]);
        test_cmd(5, text, move_down, [|buf: B| buf.text_position == 9]);
        test_cmd(5, text, move_down, [|buf: B| buf.text_position == 9]);

        test_cmd(2, text, move_down, [|buf: B| buf.text_position == 6]);
        test_cmd(6, text, move_down, [|buf: B| buf.text_position == 10]);
        test_cmd(6, text, move_down, [|buf: B| buf.text_position == 10]);
    }

    #[test]
    #[rustfmt::skip]
    fn move_down_hops() {
        let text = "0123\n567\n901";
        test_cmd(3, text, move_down, [|buf: B| buf.text_position == 7, |buf: B| buf.saved_column == 3]);
        test_cmd(5, text, move_down, [|buf: B| buf.text_position == 9]);
        test_cmd(6, text, move_down, [|buf: B| buf.text_position == 10]);
        test_cmd(6, text, move_down, [|buf: B| buf.text_position == 10]);
        test_cmd(9, text, move_down, [|buf: B| buf.text_position == 9]);
        test_cmd(10, text, move_down, [|buf: B| buf.text_position == 10]);
    }

    #[test]
    #[rustfmt::skip]
    fn move_up_simple() {
        let text = "012\n456\n890";
        test_cmd(0, text, move_up, [|buf: B| buf.text_position == 0]);
        test_cmd(1, text, move_up, [|buf: B| buf.text_position == 1]);
        test_cmd(2, text, move_up, [|buf: B| buf.text_position == 2]);

        test_cmd(4, text, move_up, [|buf: B| buf.text_position == 0]);
        test_cmd(5, text, move_up, [|buf: B| buf.text_position == 1]);
        test_cmd(6, text, move_up, [|buf: B| buf.text_position == 2]);

        test_cmd(8, text, move_up, [|buf: B| buf.text_position == 4]);
        test_cmd(9, text, move_up, [|buf: B| buf.text_position == 5]);
        test_cmd(10, text, move_up, [|buf: B| buf.text_position == 6]);
    }

    #[test]
    #[rustfmt::skip]
    fn move_up_hops() {
        let text = "0123\n567\n901";
        test_cmd(3, text, move_down, [|buf: B| buf.text_position == 7, |buf: B| buf.saved_column == 3]);
        test_cmd(5, text, move_down, [|buf: B| buf.text_position == 9]);
        test_cmd(6, text, move_down, [|buf: B| buf.text_position == 10]);
        test_cmd(6, text, move_down, [|buf: B| buf.text_position == 10]);
        test_cmd(9, text, move_down, [|buf: B| buf.text_position == 9]);
        test_cmd(10, text, move_down, [|buf: B| buf.text_position == 10]);
    }
}
