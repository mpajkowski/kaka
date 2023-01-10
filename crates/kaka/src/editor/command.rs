use std::{borrow::Cow, fmt::Debug};

use crossterm::event::{KeyCode, KeyEvent};
use kaka_core::{
    document::{Document, TransactionAttachPolicy, TransactionLeave},
    graphemes::{nth_next_grapheme_boundary, nth_prev_grapheme_boundary},
};

use crate::{
    client::composer::{Callback, PromptWidget, Widget},
    current, current_mut,
    editor::Mode,
};

use super::{
    buffer::{LineKeep, UpdateBufPositionParams},
    Buffer, Editor,
};

pub type CommandFn = fn(&mut CommandData);

pub struct CommandData<'a> {
    pub editor: &'a mut Editor,
    pub trigger: KeyEvent,
    pub count: Option<usize>,
    pub callback: Option<Callback>,
}

impl<'a> CommandData<'a> {
    fn push_widget<W: Widget + 'static>(&mut self, widget: W) {
        self.callback = Some(Box::new(move |composer| {
            composer.push_widget(widget);
        }));
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

    pub fn describe(&self) -> &str {
        &self.name
    }
}

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command").field("name", &self.name).finish()
    }
}

// commands impl
pub fn close(ctx: &mut CommandData) {
    ctx.editor.exit_code = Some(0);
}

pub fn save(ctx: &mut CommandData) {
    let (_, doc) = current!(ctx.editor);

    doc.save().unwrap();
}

pub fn switch_to_insert_mode_before(ctx: &mut CommandData) {
    switch_to_insert_mode_impl(ctx, false);
}

pub fn switch_to_insert_mode_after(ctx: &mut CommandData) {
    switch_to_insert_mode_impl(ctx, true);
}

fn switch_to_insert_mode_impl(ctx: &mut CommandData, after: bool) {
    let repeat = ctx.count.unwrap_or(1).max(1);

    let (buf, doc) = current_mut!(ctx.editor);

    buf.switch_mode(Mode::Insert);

    doc.with_transaction(
        TransactionAttachPolicy::Disallow,
        buf.text_pos(),
        |doc, tx| {
            let pos = buf.text_pos();

            if after
                && buf.update_text_position(doc, pos + 1, UpdateBufPositionParams::inserting_text())
            {
                tx.move_forward_by(1);
            }

            tx.set_repeat(repeat);

            TransactionLeave::Keep
        },
    );
}

pub fn switch_to_normal_mode(ctx: &mut CommandData) {
    let (buf, doc) = current_mut!(ctx.editor);

    let was_insert = buf.mode().is_insert();
    buf.switch_mode(Mode::Normal);

    // move one cell left when exiting insert mode and commit transaction
    if was_insert {
        buf.update_text_position(
            doc,
            buf.text_pos().saturating_sub(1),
            UpdateBufPositionParams {
                line_keep: Some(LineKeep::Min),
                ..Default::default()
            },
        );

        doc.with_transaction(
            TransactionAttachPolicy::RequireTransactionAlive,
            buf.text_pos(),
            |doc, tx| {
                tx.apply_repeats(doc.text_mut());

                TransactionLeave::Commit
            },
        );
    }
}

pub fn move_left(ctx: &mut CommandData) {
    let count = ctx.count.unwrap_or(1);
    let (buf, doc) = current_mut!(ctx.editor);

    let text = doc.text();

    let new_pos = nth_prev_grapheme_boundary(text.slice(..), buf.text_pos(), count);
    buf.update_text_position(
        doc,
        new_pos,
        UpdateBufPositionParams {
            line_keep: Some(LineKeep::Min),
            allow_on_newline: false,
            ..Default::default()
        },
    );
}

pub fn move_right(ctx: &mut CommandData) {
    let count = ctx.count.unwrap_or(1);
    let (buf, doc) = current_mut!(ctx.editor);

    let pos = buf.text_pos();
    let text = doc.text();
    let curr_line = buf.line_idx();
    let line_start = buf.line_char();
    let line = text.line(curr_line);

    let new_pos =
        line_start + nth_next_grapheme_boundary(line, pos.saturating_sub(line_start), count);

    buf.update_text_position(
        doc,
        new_pos,
        UpdateBufPositionParams {
            line_keep: Some(LineKeep::Max),
            allow_on_newline: false,
            ..Default::default()
        },
    );
}

pub fn move_up(ctx: &mut CommandData) {
    goto_line_impl(ctx, GotoLine::Offset(-(ctx.count.unwrap_or(1) as i128)));
}

pub fn move_down(ctx: &mut CommandData) {
    goto_line_impl(ctx, GotoLine::Offset(ctx.count.unwrap_or(1) as i128));
}

pub fn goto_line_default_top(ctx: &mut CommandData) {
    let line = ctx.count.and_then(|c| c.checked_sub(1)).unwrap_or(0);

    goto_line_impl(ctx, GotoLine::Fixed(line));
}

pub fn goto_line_default_bottom(ctx: &mut CommandData) {
    let (_, doc) = current_mut!(ctx.editor);

    let line = ctx
        .count
        .and_then(|c| c.checked_sub(1))
        .unwrap_or_else(|| doc.text().len_lines().saturating_sub(1));

    goto_line_impl(ctx, GotoLine::Fixed(line));
}

#[derive(Debug, Clone, Copy)]
enum GotoLine {
    Fixed(usize),
    Offset(i128),
}

impl GotoLine {
    fn to_line(self, buf: &Buffer, doc: &Document) -> usize {
        let text = doc.text();
        let limit = text.len_lines().saturating_sub(1);

        match self {
            Self::Fixed(line) => line.min(limit),
            Self::Offset(offset) => {
                let curr_line_start = buf.line_idx();

                ((curr_line_start as i128).saturating_add(offset)).clamp(0, limit as i128) as usize
            }
        }
    }
}

fn goto_line_impl(ctx: &mut CommandData, goto_line: GotoLine) {
    let (buf, doc) = current_mut!(ctx.editor);

    let text = doc.text();

    let goto_line_idx = goto_line.to_line(buf, doc);
    let goto_line_start = text.line_to_char(goto_line_idx);
    let goto_line_end = text.line_to_char(goto_line_idx + 1).saturating_sub(1);

    let mut new_pos = (goto_line_start + buf.saved_column()).min(goto_line_end);

    new_pos = new_pos.max(goto_line_start);

    buf.update_text_position(
        doc,
        new_pos,
        UpdateBufPositionParams {
            update_saved_column: false,
            allow_on_newline: false,
            line_keep: None,
        },
    );
}

pub fn delete_line(ctx: &mut CommandData) {
    let (buf, doc) = current_mut!(ctx.editor);

    let text = doc.text_mut();
    let pos = buf.text_pos();

    let line_idx = text.char_to_line(pos);
    let line_start = text.line_to_char(line_idx);
    let line_end = text.line_to_char(line_idx + 1);

    text.remove(line_start..line_end);
}

pub fn remove_char(ctx: &mut CommandData) {
    let (buf, doc) = current_mut!(ctx.editor);

    doc.with_transaction(
        TransactionAttachPolicy::Disallow,
        buf.text_pos(),
        |doc, tx| {
            let text = doc.text_mut();

            let current_line_idx = buf.line_idx();
            let current_line_start = buf.line_char();
            let current_line_end = text.line_to_char(current_line_idx + 1);
            let pos = buf.text_pos();

            if (current_line_start..current_line_end).contains(&pos)
                && text.try_remove(pos..=pos).is_ok()
            {
                if pos == current_line_end.saturating_sub(2) {
                    buf.update_text_position(doc, pos, Default::default());
                }

                tx.delete(1);
                return TransactionLeave::Commit;
            }

            TransactionLeave::Rollback
        },
    );
}

pub fn insert_mode_on_key(ctx: &mut CommandData, event: KeyEvent) {
    let (buf, doc) = current_mut!(ctx.editor);

    debug_assert!(matches!(buf.mode(), Mode::Insert));

    let mut pos = buf.text_pos();
    doc.with_transaction(
        TransactionAttachPolicy::RequireTransactionAlive,
        pos,
        |doc, tx| {
            let text = doc.text_mut();

            match event.code {
                KeyCode::Char(c) => {
                    text.insert_char(pos, c);
                    tx.insert_char(c);

                    pos += 1;
                }
                KeyCode::Backspace => {
                    if pos > 0 {
                        text.remove(pos - 1..pos);

                        tx.move_backward_by(1);
                        tx.delete(1);
                        pos -= 1;
                    }
                }
                KeyCode::Enter => {
                    text.insert_char(pos, '\n');
                    tx.insert_char('\n');

                    pos += 1;
                }
                KeyCode::Left => {
                    if pos > 0 {
                        tx.move_backward_by(1);
                        pos -= 1;
                    }
                }
                KeyCode::Right => {
                    if pos < text.len_chars() - 1 {
                        pos += 1;
                        tx.move_forward_by(1);
                    }
                }
                _ => { /* TODO */ }
            };

            buf.update_text_position(doc, pos, UpdateBufPositionParams::inserting_text());

            TransactionLeave::Keep
        },
    );
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

pub fn undo(ctx: &mut CommandData) {
    let (buf, doc) = current_mut!(ctx.editor);

    if let Some(pos) = doc.undo() {
        buf.update_text_position(doc, pos, Default::default());
    }
}

pub fn redo(ctx: &mut CommandData) {
    let (buf, doc) = current_mut!(ctx.editor);

    if let Some(pos) = doc.redo() {
        buf.update_text_position(doc, pos, Default::default());
    }
}

pub fn command_mode(ctx: &mut CommandData) {
    ctx.push_widget(PromptWidget::new(':'));
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

        let mut data = CommandData {
            editor: &mut editor,
            trigger: KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
            count: Some(1),
            callback: None,
        };

        command(&mut data);

        let (buf, _) = current!(data.editor);

        for check in checks_buffer {
            assert!(check(buf), "Buffer assert failed: {buf:#?}");
        }
    }

    // to save characters :P
    type B<'a> = &'a Buffer;

    #[test]
    #[rustfmt::skip]
    fn move_left_prevented_on_pos_0() {
        test_cmd(0, "kakaka\n", move_left, [|buf: B| buf.text_pos() == 0, |buf: B| buf.saved_column() == 0]);
        test_cmd(7, "kakaka\nkaka", move_left, [|buf: B| buf.text_pos() == 7]);
    }

    #[test]
    #[rustfmt::skip]
    fn move_left_doable_until_newline() {
        test_cmd(3, "kaka\n", move_left, [|buf: B| buf.text_pos() == 2, |buf: B| buf.saved_column() == 2]);
        test_cmd(2, "kaka\n", move_left, [|buf: B| buf.text_pos() == 1, |buf: B| buf.saved_column() == 1]);
        test_cmd(1, "kaka\n", move_left, [|buf: B| buf.text_pos() == 0, |buf: B| buf.saved_column() == 0]);
    }

    #[test]
    #[rustfmt::skip]
    fn move_right_doable_until_newline() {
        test_cmd(0, "kaka\n", move_right, [|buf: B| buf.text_pos() == 1, |buf: B| buf.saved_column() == 1]);
        test_cmd(1, "kaka\n", move_right, [|buf: B| buf.text_pos() == 2, |buf: B| buf.saved_column() == 2]);
        test_cmd(2, "kaka\n", move_right, [|buf: B| buf.text_pos() == 3, |buf: B| buf.saved_column() == 3]);
        test_cmd(3, "kaka", move_right, [|buf: B| buf.text_pos() == 3, |buf: B| buf.saved_column() == 3]);
        test_cmd(3, "kaka\n", move_right, [|buf: B| buf.text_pos() == 3, |buf: B| buf.saved_column() == 3]);
        test_cmd(5, "kaka\nkaka", move_right, [|buf: B| buf.text_pos() == 6, |buf: B| buf.saved_column() == 1]);
        test_cmd(6, "kaka\nkaka", move_right, [|buf: B| buf.text_pos() == 7, |buf: B| buf.saved_column() == 2]);
        test_cmd(7, "kaka\nkaka", move_right, [|buf: B| buf.text_pos() == 8, |buf: B| buf.saved_column() == 3]);
        test_cmd(7, "kaka\nkaka\n", move_right, [|buf: B| buf.text_pos() == 8, |buf: B| buf.saved_column() == 3]);
    }

    #[test]
    #[rustfmt::skip]
    fn move_down_simple() {
        let text = "012\n456\n890";
        test_cmd(0, text, move_down, [|buf: B| buf.text_pos() == 4]);
        test_cmd(4, text, move_down, [|buf: B| buf.text_pos() == 8]);
        test_cmd(8, text, move_down, [|buf: B| buf.text_pos() == 8]);

        test_cmd(1, text, move_down, [|buf: B| buf.text_pos() == 5]);
        test_cmd(5, text, move_down, [|buf: B| buf.text_pos() == 9]);
        test_cmd(5, text, move_down, [|buf: B| buf.text_pos() == 9]);

        test_cmd(2, text, move_down, [|buf: B| buf.text_pos() == 6]);
        test_cmd(6, text, move_down, [|buf: B| buf.text_pos() == 10]);
        test_cmd(6, text, move_down, [|buf: B| buf.text_pos() == 10]);
    }

    #[test]
    #[rustfmt::skip]
    fn move_up_simple() {
        let text = "012\n456\n890";
        test_cmd(0, text, move_up, [|buf: B| buf.text_pos() == 0]);
        test_cmd(1, text, move_up, [|buf: B| buf.text_pos() == 1]);
        test_cmd(2, text, move_up, [|buf: B| buf.text_pos() == 2]);

        test_cmd(4, text, move_up, [|buf: B| buf.text_pos() == 0]);
        test_cmd(5, text, move_up, [|buf: B| buf.text_pos() == 1]);
        test_cmd(6, text, move_up, [|buf: B| buf.text_pos() == 2]);

        test_cmd(8, text, move_up, [|buf: B| buf.text_pos() == 4]);
        test_cmd(9, text, move_up, [|buf: B| buf.text_pos() == 5]);
        test_cmd(10, text, move_up, [|buf: B| buf.text_pos() == 6]);
    }

    #[test]
    #[rustfmt::skip]
    fn move_down_hops() {
        let text = "0123\n567\n901";
        test_cmd(3, text, move_down, [|buf: B| buf.text_pos() == 7, |buf: B| buf.saved_column() == 3]);
        test_cmd(5, text, move_down, [|buf: B| buf.text_pos() == 9]);
        test_cmd(6, text, move_down, [|buf: B| buf.text_pos() == 10]);
        test_cmd(6, text, move_down, [|buf: B| buf.text_pos() == 10]);
        test_cmd(9, text, move_down, [|buf: B| buf.text_pos() == 9]);
        test_cmd(10, text, move_down, [|buf: B| buf.text_pos() == 10]);
        let text = "0123\n567\n901\n\n\n";
        test_cmd(3, text, move_down, [|buf: B| buf.text_pos() == 7, |buf: B| buf.saved_column() == 3]);
        test_cmd(5, text, move_down, [|buf: B| buf.text_pos() == 9]);
        test_cmd(6, text, move_down, [|buf: B| buf.text_pos() == 10]);
        test_cmd(6, text, move_down, [|buf: B| buf.text_pos() == 10]);
        test_cmd(9, text, move_down, [|buf: B| buf.text_pos() == 13]);
        test_cmd(10, text, move_down, [|buf: B| buf.text_pos() == 13]);
        test_cmd(11, text, move_down, [|buf: B| buf.text_pos() == 13]);
        test_cmd(13, text, move_down, [|buf: B| buf.text_pos() == 14]);
        test_cmd(14, text, move_down, [|buf: B| buf.text_pos() == 15]);
    }
}
