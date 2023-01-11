use kaka_core::{
    document::{TransactionAttachPolicy, TransactionLeave},
    graphemes::{nth_next_grapheme_boundary, nth_prev_grapheme_boundary},
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

#[derive(Debug)]
enum Switch {
    Inplace,
    After,
    LineStart,
    LineEnd,
}

pub fn switch_to_insert_mode_inplace(ctx: &mut CommandData) {
    switch_to_insert_mode_impl(ctx, Switch::Inplace);
}

pub fn switch_to_insert_mode_after(ctx: &mut CommandData) {
    switch_to_insert_mode_impl(ctx, Switch::After);
}

pub fn switch_to_insert_mode_line_start(ctx: &mut CommandData) {
    switch_to_insert_mode_impl(ctx, Switch::LineStart);
}

pub fn switch_to_insert_mode_line_end(ctx: &mut CommandData) {
    switch_to_insert_mode_impl(ctx, Switch::LineEnd);
}

fn switch_to_insert_mode_impl(ctx: &mut CommandData, switch: Switch) {
    let repeat = ctx.count.unwrap_or(1).max(1);

    let (buf, doc) = current_mut!(ctx.editor);

    buf.switch_mode(Mode::Insert);

    let pos = buf.text_pos();

    doc.with_transaction(TransactionAttachPolicy::Disallow, pos, |doc, tx| {
        let text = doc.text().slice(..);

        let new_pos = match switch {
            Switch::Inplace => pos,
            Switch::After => nth_next_grapheme_boundary(text, pos, 1),
            Switch::LineStart => buf.line_char(),
            Switch::LineEnd => {
                let eol = text
                    .try_line_to_char(buf.line_idx() + 1)
                    .unwrap_or_else(|_| text.len_chars());

                nth_prev_grapheme_boundary(text, eol, 1)
            }
        };

        if buf.update_text_position(doc, new_pos, UpdateBufPositionParams::inserting_text()) {
            tx.move_to(new_pos);
        }

        tx.set_repeat(repeat);

        TransactionLeave::Keep
    });
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

#[cfg(test)]
mod test {
    use super::super::test::*;
    use super::*;

    #[test]
    fn transaction_opened() {
        test_cmd(0, "", switch_to_insert_mode_after, |_: B, doc: D| {
            assert!(doc.transaction_active());
        });
        test_cmd(0, "", switch_to_insert_mode_inplace, |_: B, doc: D| {
            assert!(doc.transaction_active());
        });
        test_cmd(0, "", switch_to_insert_mode_line_end, |_: B, doc: D| {
            assert!(doc.transaction_active());
        });
        test_cmd(0, "", switch_to_insert_mode_line_start, |_: B, doc: D| {
            assert!(doc.transaction_active());
        });
    }
}
