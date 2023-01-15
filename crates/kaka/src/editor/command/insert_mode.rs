use crossterm::event::{KeyCode, KeyEvent};
use kaka_core::{document::TransactionLeave, transaction::Transaction};

use crate::{
    current_mut,
    editor::{buffer::UpdateBufPositionParams, ModeKind},
};

use super::CommandData;

pub fn insert_mode_on_key(ctx: &mut CommandData, event: KeyEvent) {
    let (buf, doc) = current_mut!(ctx.editor);

    debug_assert!(matches!(buf.mode(), ModeKind::Insert));

    doc.with_transaction(|doc, insert_tx| {
        let text = doc.text_mut();

        let pos = buf.text_pos();
        let mut tx = Transaction::new(text, pos);

        match event.code {
            KeyCode::Char(c) => {
                tx.insert_char(c);
            }
            KeyCode::Backspace => {
                if pos > 0 {
                    tx.move_backward_by(1);
                    tx.delete(1);
                }
            }
            KeyCode::Enter => {
                tx.insert_char('\n');
            }
            KeyCode::Left => {
                if pos > 0 {
                    tx.move_backward_by(1);
                }
            }
            KeyCode::Right => {
                if pos < text.len_chars() - 1 {
                    tx.move_forward_by(1);
                }
            }
            _ => { /* TODO */ }
        };

        let pos = tx.apply(text);
        buf.update_text_position(doc, pos, UpdateBufPositionParams::inserting_text());

        insert_tx.merge(tx);

        TransactionLeave::Keep
    });
}
