use buffer::Buffer;
use kaka_core::{
    document::{Document, TransactionLeave},
    graphemes::next_grapheme_boundary,
};

use crate::{
    current_mut,
    editor::{
        buffer::{self, UpdateBufPositionParams},
        ModeKind,
    },
};

use super::CommandData;

pub fn kill_line(ctx: &mut CommandData) {
    let (buf, doc) = current_mut!(ctx.editor);

    let text = doc.text();
    let line_start = buf.line_char();
    let line_end = text.line_to_char(buf.line_idx() + 1);

    doc.with_new_transaction(buf.text_pos(), |doc, tx| {
        tx.move_to(line_start);
        tx.delete(line_end - line_start);

        tx.apply(doc.text_mut());

        TransactionLeave::Commit
    });
}

pub fn kill(ctx: &mut CommandData) {
    let (buf, doc) = current_mut!(ctx.editor);

    if let Some(selection) = buf.selection().map(|s| s.range()) {
        kill_selection(buf, doc, selection);
        buf.switch_mode(ModeKind::Normal);
    } else {
        kill_char(buf, doc);
    }
}

fn kill_selection(buf: &mut Buffer, doc: &mut Document, (start, mut end): (usize, usize)) {
    doc.with_new_transaction(start, |doc, tx| {
        let mut tmp = doc.text().clone();
        end = next_grapheme_boundary(tmp.slice(..), end);

        tx.delete(end - start);
        tx.apply(&mut tmp);

        if let Some(new_pos) = buf.update_text_position(
            &tmp,
            start,
            UpdateBufPositionParams {
                line_keep: false,
                allow_on_newline: false,
                ..Default::default()
            },
        ) {
            tx.move_to(new_pos);
        }

        tx.apply(doc.text_mut());

        TransactionLeave::Commit
    });
}

fn kill_char(buf: &mut Buffer, doc: &mut Document) {
    let pos = buf.text_pos();

    doc.with_new_transaction(pos, |doc, tx| {
        if matches!(
            doc.text().line(buf.line_idx()).get_char(0),
            Some('\n') | None
        ) {
            return TransactionLeave::Rollback;
        }

        tx.delete(1);
        let mut tmp = doc.text().clone();
        tx.apply(&mut tmp);

        if let Some(new_pos) = buf.update_text_position(
            &tmp,
            pos,
            UpdateBufPositionParams {
                line_keep: true,
                allow_on_newline: false,
                ..Default::default()
            },
        ) {
            tx.move_to(new_pos);
            log::info!("Pos: {}", buf.text_pos());
        }

        tx.apply(doc.text_mut());

        TransactionLeave::Commit
    });
}
