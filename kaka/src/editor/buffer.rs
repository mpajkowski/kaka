use anyhow::{ensure, Context, Result};
use kaka_core::document::{Document, DocumentId};

use std::{
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::Mode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BufferId(NonZeroUsize);

impl BufferId {
    pub const MAX: Self = Self(unsafe { NonZeroUsize::new_unchecked(usize::MAX) });

    pub fn next() -> Self {
        pub static IDS: AtomicUsize = AtomicUsize::new(1);

        let next = NonZeroUsize::new(IDS.fetch_add(1, Ordering::SeqCst))
            .expect("BufferId counter overflowed");

        Self(next)
    }
}

#[derive(Debug)]
pub struct Buffer {
    id: BufferId,
    document_id: DocumentId,
    avail_modes: Vec<Mode>,
    current_mode: usize,
    immortal: bool,
    pub text_position: usize,
    pub saved_column: usize,
}

impl Buffer {
    pub fn new_text(pos: usize, document: &Document) -> Result<Self> {
        Self::new(
            pos,
            [Mode::Normal, Mode::Xd, Mode::Insert],
            document,
            &Mode::Normal,
            false,
        )
    }

    pub fn new_logging(document: &Document) -> Self {
        Self::new(0, vec![Mode::Normal], document, &Mode::Normal, true).unwrap()
    }

    pub fn new(
        pos: usize,
        avail_modes: impl IntoIterator<Item = Mode>,
        document: &Document,
        start_mode: &Mode,
        immortal: bool,
    ) -> Result<Self> {
        let text = document.text();

        ensure!(
            pos == 0 || text.get_char(pos).is_some(),
            "Start position {pos} is out of bounds"
        );

        let mut this = Self {
            id: BufferId::next(),
            document_id: document.id(),
            avail_modes: avail_modes.into_iter().collect(),
            current_mode: 0,
            text_position: pos,
            saved_column: 0,
            immortal,
        };

        this.set_mode_impl(start_mode.name())?;
        this.update_saved_column(document);

        Ok(this)
    }

    pub fn update_saved_column(&mut self, doc: &Document) {
        let text = doc.text();
        let pos = self.text_position;

        let start_line_idx = text.char_to_line(pos);
        let start_line_pos = text.line_to_char(start_line_idx);
        self.saved_column = pos - start_line_pos;
    }

    pub const fn id(&self) -> BufferId {
        self.id
    }

    pub const fn document_id(&self) -> DocumentId {
        self.document_id
    }

    pub fn mode(&self) -> &Mode {
        &self.avail_modes[self.current_mode]
    }

    pub fn switch_mode(&mut self, mode: &str) {
        // ignore error for now
        self.set_mode_impl(mode).ok();
    }

    pub const fn immortal(&self) -> bool {
        self.immortal
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

#[cfg(test)]
mod test {
    use kaka_core::ropey::Rope;

    use super::*;

    #[test]
    fn start_position() {
        let mut document = Document::new_scratch();
        *document.text_mut() = Rope::from("kaka\n");

        let buffer = Buffer::new_text(0, &document).unwrap();
        assert_eq!(buffer.text_position, 0);
        assert_eq!(buffer.saved_column, 0);

        let buffer = Buffer::new_text(1, &document).unwrap();
        assert_eq!(buffer.text_position, 1);
        assert_eq!(buffer.saved_column, 1);

        let buffer = Buffer::new_text(2, &document).unwrap();
        assert_eq!(buffer.text_position, 2);
        assert_eq!(buffer.saved_column, 2);

        let buffer = Buffer::new_text(3, &document).unwrap();
        assert_eq!(buffer.text_position, 3);
        assert_eq!(buffer.saved_column, 3);

        assert!(
            Buffer::new_text(5, &document).is_err(),
            "Created buffer with position set out of document bounds"
        );
    }

    #[test]
    fn mode_switch() {
        let modes = [Mode::Normal, Mode::Insert];

        let document = Document::new_scratch();
        let mut buffer = Buffer::new(0, modes, &document, &Mode::Normal, false).unwrap();
        assert!(matches!(buffer.mode(), &Mode::Normal));

        buffer.switch_mode("insert");
        assert!(buffer.mode().is_insert());
    }
}
