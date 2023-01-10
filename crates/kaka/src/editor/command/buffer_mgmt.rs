use kaka_core::document::Document;

use crate::{current, editor::Buffer};

use super::CommandData;

pub fn buffer_next(ctx: &mut CommandData) {
    let curr = ctx.editor.current;

    let mut iter = ctx.editor.buffers.keys();

    let next = iter
        .clone()
        .find(|id| **id > curr)
        .or_else(|| iter.next())
        .unwrap();

    ctx.editor.current = *next;
}

pub fn buffer_prev(ctx: &mut CommandData) {
    let curr = ctx.editor.current;

    let mut iter = ctx.editor.buffers.keys().rev();

    let prev = iter
        .clone()
        .find(|id| **id < curr)
        .or_else(|| iter.next())
        .unwrap();

    ctx.editor.current = *prev;
}

pub fn buffer_create(ctx: &mut CommandData) {
    let scratch = Document::new_scratch();
    let buffer = Buffer::new_text(0, &scratch).unwrap();

    let doc_id = scratch.id();
    let buf_id = buffer.id();

    ctx.editor.documents.insert(doc_id, scratch);
    ctx.editor.buffers.insert(buf_id, buffer);
    ctx.editor.current = buf_id;
}

pub fn buffer_kill(ctx: &mut CommandData) {
    let immortal = ctx
        .editor
        .buffers
        .get(&ctx.editor.current)
        .unwrap()
        .immortal();

    if !immortal {
        ctx.editor.buffers.remove(&ctx.editor.current);

        if ctx.editor.buffers.is_empty() {
            buffer_create(ctx);
        } else {
            buffer_prev(ctx);
        }
    }
}

// commands impl
pub fn close(ctx: &mut CommandData) {
    ctx.editor.exit_code = Some(0);
}

pub fn save(ctx: &mut CommandData) {
    let (_, doc) = current!(ctx.editor);

    doc.save().unwrap();
}
