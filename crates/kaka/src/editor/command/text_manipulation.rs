use kaka_core::document::TransactionLeave;

use crate::{current_mut, editor::buffer::UpdateBufPositionParams};

use super::CommandData;

pub fn delete_line(ctx: &mut CommandData) {
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

pub fn remove_char(ctx: &mut CommandData) {
    let (buf, doc) = current_mut!(ctx.editor);

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
