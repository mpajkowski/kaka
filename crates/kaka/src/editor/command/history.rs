use crate::current_mut;

use super::CommandData;

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
