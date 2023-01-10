//! Contains macros that allow to access nested fields in the `Editor` struct.
//!
//! Inspired by helix

#[macro_export]
macro_rules! current {
    ($editor:expr) => {{
        let buffer = $editor.buffers.get(&$editor.current).unwrap();
        let doc_id = buffer.document_id();
        let document = $editor.documents.get(&doc_id).unwrap();

        (buffer, document)
    }};
}

#[macro_export]
macro_rules! current_mut {
    ($editor:expr) => {{
        let buffer = $editor.buffers.get_mut(&$editor.current).unwrap();
        let doc_id = buffer.document_id();
        let document = $editor.documents.get_mut(&doc_id).unwrap();

        (buffer, document)
    }};
}
