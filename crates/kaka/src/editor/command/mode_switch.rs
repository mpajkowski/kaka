use kaka_core::{
    document::{TransactionAttachPolicy, TransactionLeave},
    graphemes::nth_next_grapheme_boundary,
};

use crate::{
    client::composer::PromptWidget,
    current_mut,
    editor::{
        buffer::{LineKeep, UpdateBufPositionParams},
        Mode,
    },
};

use super::CommandData;

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
            if after {
                let line_char = buf.line_char();
                let pos = line_char
                    + nth_next_grapheme_boundary(doc.text().line(buf.line_idx()), line_char, 1);
                buf.update_text_position(doc, pos, UpdateBufPositionParams::inserting_text());
                tx.move_forward_by(pos);
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

pub fn command_mode(ctx: &mut CommandData) {
    ctx.push_widget(PromptWidget::new(':'));
}
