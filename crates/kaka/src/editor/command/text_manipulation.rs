use kaka_core::document::{TransactionAttachPolicy, TransactionLeave};

use crate::current_mut;

use super::CommandData;

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
