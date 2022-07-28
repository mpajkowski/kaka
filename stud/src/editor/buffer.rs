use stud_core::{Document, DocumentId};

use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use super::{Editor, Keymap, Mode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BufferId(NonZeroUsize);

impl BufferId {
    pub fn inner(self) -> usize {
        self.0.get()
    }

    pub fn next() -> Self {
        pub static IDS: AtomicUsize = AtomicUsize::new(1);
        Self(
            NonZeroUsize::new(IDS.fetch_add(1, Ordering::SeqCst))
                .expect("BufferId counter is messed"),
        )
    }
}

impl Default for BufferId {
    fn default() -> Self {
        Self(unsafe { NonZeroUsize::new_unchecked(1) })
    }
}

pub struct Buffer {
    id: BufferId,
    document_id: DocumentId,
    mode: Arc<Mode>,
}

impl Buffer {
    pub fn new(mode: Arc<Mode>, document: &Document) -> Self {
        Self {
            id: BufferId::next(),
            document_id: document.id(),
            mode,
        }
    }

    #[inline]
    pub fn id(&self) -> BufferId {
        self.id
    }

    #[inline]
    pub fn document_id(&self) -> DocumentId {
        self.document_id
    }

    pub fn document<'a>(&self, editor: &'a Editor) -> &'a Document {
        editor
            .documents
            .get(&self.document_id())
            .expect("document removed from editor")
    }

    pub fn document_mut<'a>(&self, editor: &'a mut Editor) -> &'a mut Document {
        editor
            .documents
            .get_mut(&self.document_id())
            .expect("document removed from editor")
    }

    pub fn keymap(&self) -> &Keymap {
        self.mode.keymap()
    }
}
