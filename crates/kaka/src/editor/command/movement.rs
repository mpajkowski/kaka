use kaka_core::{
    document::Document,
    graphemes::{nth_next_grapheme_boundary, nth_prev_grapheme_boundary},
};

use crate::{
    current_mut,
    editor::{
        buffer::{LineKeep, UpdateBufPositionParams},
        Buffer,
    },
};

use super::CommandData;

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

#[cfg(test)]
mod test {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use kaka_core::ropey::Rope;

    use crate::{
        current,
        editor::{Buffer, Editor},
    };

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
