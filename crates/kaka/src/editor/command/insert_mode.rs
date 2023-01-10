use crossterm::event::{KeyCode, KeyEvent};
use kaka_core::document::{TransactionAttachPolicy, TransactionLeave};

use crate::{
    current_mut,
    editor::{buffer::UpdateBufPositionParams, Mode},
};

use super::CommandData;

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
