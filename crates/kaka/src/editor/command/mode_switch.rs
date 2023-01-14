use kaka_core::{document::TransactionLeave, graphemes::nth_next_grapheme_boundary};

use crate::{
    client::composer::PromptWidget,
    current_mut,
    editor::{buffer::UpdateBufPositionParams, Mode},
};

use super::CommandData;

#[derive(Debug, Clone, Copy)]
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
    use Switch::*;
    let repeat = ctx.count.unwrap_or(1).max(1);

    let (buf, doc) = current_mut!(ctx.editor);

    buf.switch_mode(Mode::Insert);

    let pos = buf.text_pos();

    let line = doc.text().line(buf.line_idx());
    let line_char = buf.line_char();
    let line_len = line.len_chars();

    let approx_new_pos = match switch {
        Inplace => pos,
        LineStart => line_char,
        After => line_char + nth_next_grapheme_boundary(line, pos - line_char, 1),
        LineEnd => line_char + line_len,
    };

    let insert_after_cursor = matches!(switch, After | LineEnd);

    let params = UpdateBufPositionParams {
        update_saved_column: true,
        line_keep: insert_after_cursor,
        allow_on_newline: insert_after_cursor,
    };

    let pos = buf
        .update_text_position(doc, approx_new_pos, params)
        .unwrap_or(approx_new_pos);

    doc.open_transaction(pos);
    doc.with_transaction(|_, tx| {
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
                line_keep: true,
                ..Default::default()
            },
        );

        doc.with_transaction(|doc, tx| {
            tx.apply_repeats(doc.text_mut());

            TransactionLeave::Commit
        });
    }
}

pub fn command_mode(ctx: &mut CommandData) {
    ctx.push_widget(PromptWidget::new(":", |this, ctx| {
        let command_name = this.text();
        ctx.invoke_command_by_name(command_name);
    }));
}

#[cfg(test)]
mod test {
    use super::super::test::*;
    use super::*;

    #[test]
    fn enter_insert_mode_transaction_opened() {
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

    #[test]
    fn enter_insert_mode_after_position() {
        let text = "012\n4567\n9AB\n";

        test_cmd(0, text, switch_to_insert_mode_after, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 1);
        });
        test_cmd(1, text, switch_to_insert_mode_after, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 2);
        });
        test_cmd(2, text, switch_to_insert_mode_after, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 3);
        });

        test_cmd(4, text, switch_to_insert_mode_after, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 5);
        });
        test_cmd(5, text, switch_to_insert_mode_after, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 6);
        });
        test_cmd(6, text, switch_to_insert_mode_after, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 7);
        });
        test_cmd(7, text, switch_to_insert_mode_after, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 8);
        });

        test_cmd(9, text, switch_to_insert_mode_after, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 10);
        });
        test_cmd(10, text, switch_to_insert_mode_after, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 11);
        });
        test_cmd(11, text, switch_to_insert_mode_after, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 12);
        });
    }

    #[test]
    fn enter_insert_mode_inplace_position() {
        let text = "012\n4567\n9AB\n";

        test_cmd(0, text, switch_to_insert_mode_inplace, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 0);
        });
        test_cmd(1, text, switch_to_insert_mode_inplace, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 1);
        });
        test_cmd(2, text, switch_to_insert_mode_inplace, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 2);
        });

        test_cmd(4, text, switch_to_insert_mode_inplace, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 4);
        });
        test_cmd(5, text, switch_to_insert_mode_inplace, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 5);
        });
        test_cmd(6, text, switch_to_insert_mode_inplace, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 6);
        });
        test_cmd(7, text, switch_to_insert_mode_inplace, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 7);
        });

        test_cmd(9, text, switch_to_insert_mode_inplace, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 9);
        });
        test_cmd(10, text, switch_to_insert_mode_inplace, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 10);
        });
        test_cmd(11, text, switch_to_insert_mode_inplace, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 11);
        });
    }

    #[test]
    fn enter_insert_mode_line_start_position() {
        let text = "012\n4567\n9AB\n";

        test_cmd(0, text, switch_to_insert_mode_line_start, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 0);
        });
        test_cmd(1, text, switch_to_insert_mode_line_start, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 0);
        });
        test_cmd(2, text, switch_to_insert_mode_line_start, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 0);
        });

        test_cmd(4, text, switch_to_insert_mode_line_start, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 4);
        });
        test_cmd(5, text, switch_to_insert_mode_line_start, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 4);
        });
        test_cmd(6, text, switch_to_insert_mode_line_start, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 4);
        });
        test_cmd(7, text, switch_to_insert_mode_line_start, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 4);
        });

        test_cmd(9, text, switch_to_insert_mode_line_start, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 9);
        });
        test_cmd(
            10,
            text,
            switch_to_insert_mode_line_start,
            |buf: B, _: D| {
                assert_eq!(buf.text_pos(), 9);
            },
        );
        test_cmd(
            11,
            text,
            switch_to_insert_mode_line_start,
            |buf: B, _: D| {
                assert_eq!(buf.text_pos(), 9);
            },
        );
    }

    #[test]
    fn enter_insert_mode_line_end_position() {
        let text = "012\n4567\n9AB\n";

        test_cmd(0, text, switch_to_insert_mode_line_end, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 3);
        });
        test_cmd(1, text, switch_to_insert_mode_line_end, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 3);
        });
        test_cmd(2, text, switch_to_insert_mode_line_end, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 3);
        });

        test_cmd(4, text, switch_to_insert_mode_line_end, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 8);
        });
        test_cmd(5, text, switch_to_insert_mode_line_end, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 8);
        });
        test_cmd(6, text, switch_to_insert_mode_line_end, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 8);
        });
        test_cmd(7, text, switch_to_insert_mode_line_end, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 8);
        });

        test_cmd(9, text, switch_to_insert_mode_line_end, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 12);
        });
        test_cmd(10, text, switch_to_insert_mode_line_end, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 12);
        });
        test_cmd(11, text, switch_to_insert_mode_line_end, |buf: B, _: D| {
            assert_eq!(buf.text_pos(), 12);
        });
    }
}
