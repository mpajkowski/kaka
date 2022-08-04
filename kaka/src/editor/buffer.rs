use anyhow::{Context, Result};
use kaka_core::{Document, DocumentId};

use std::{
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::Mode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BufferId(NonZeroUsize);

impl BufferId {
    pub fn next() -> Self {
        pub static IDS: AtomicUsize = AtomicUsize::new(1);
        Self(
            NonZeroUsize::new(IDS.fetch_add(1, Ordering::SeqCst))
                .expect("BufferId counter is messed"),
        )
    }
}

pub struct Buffer {
    id: BufferId,
    document_id: DocumentId,
    avail_modes: Vec<Mode>,
    current_mode: usize,
}

impl Buffer {
    pub fn new_text_buffer(document: &Document) -> Self {
        Self::new([Mode::Xd, Mode::Insert], document, &Mode::Xd).unwrap()
    }

    pub fn new(
        avail_modes: impl IntoIterator<Item = Mode>,
        document: &Document,
        start_mode: &Mode,
    ) -> Result<Self> {
        let mut this = Self {
            id: BufferId::next(),
            document_id: document.id(),
            avail_modes: avail_modes.into_iter().collect(),
            current_mode: 0,
        };
        this.set_mode_impl(start_mode.name())?;

        Ok(this)
    }

    #[inline]
    pub const fn id(&self) -> BufferId {
        self.id
    }

    #[inline]
    pub const fn document_id(&self) -> DocumentId {
        self.document_id
    }

    pub fn mode(&self) -> &Mode {
        &self.avail_modes[self.current_mode]
    }

    pub fn set_mode(&mut self, mode: &str) {
        // ignore error for now
        self.set_mode_impl(mode).ok();
    }

    fn set_mode_impl(&mut self, mode: &str) -> Result<()> {
        let mode_pos = self
            .avail_modes
            .iter()
            .position(|m| m.name() == mode)
            .with_context(|| format!("Buffer is not capable to enter {mode}"))?;

        self.current_mode = mode_pos;

        Ok(())
    }
}
