use anyhow::{ensure, Context, Result};
use kaka_core::{
    document::{Document, DocumentId},
    graphemes::nth_next_grapheme_boundary,
};

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
    saved_column: usize,
    text_pos: usize,
    line_idx: usize,
    line_char: usize,
}

impl Buffer {
    pub fn new_text(pos: usize, document: &Document) -> Result<Self> {
        Self::new(
            pos,
            [Mode::Normal, Mode::Xd, Mode::Insert],
            document,
            Mode::Normal,
            false,
        )
    }

    pub fn new_logging(document: &Document) -> Self {
        Self::new(0, vec![Mode::Normal], document, Mode::Normal, true).unwrap()
    }

    pub fn new(
        pos: usize,
        avail_modes: impl IntoIterator<Item = Mode>,
        document: &Document,
        start_mode: Mode,
        immortal: bool,
    ) -> Result<Self> {
        let text = document.text();

        ensure!(
            pos <= text.len_chars().saturating_sub(1),
            "Start position {pos} is out of bounds"
        );

        let mut this = Self {
            id: BufferId::next(),
            document_id: document.id(),
            avail_modes: avail_modes.into_iter().collect(),
            current_mode: 0,
            text_pos: 0,
            saved_column: 0,
            line_idx: 0,
            line_char: 0,
            immortal,
        };

        this.set_mode_impl(start_mode)?;
        this.update_text_position(document, pos, Default::default());

        Ok(this)
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

    pub fn switch_mode(&mut self, mode: Mode) {
        // ignore error for now
        self.set_mode_impl(mode).ok();
    }

    pub const fn immortal(&self) -> bool {
        self.immortal
    }

    pub const fn saved_column(&self) -> usize {
        self.saved_column
    }

    pub const fn text_pos(&self) -> usize {
        self.text_pos
    }

    pub const fn line_idx(&self) -> usize {
        self.line_idx
    }

    pub const fn line_char(&self) -> usize {
        self.line_char
    }

    pub fn update_text_position(
        &mut self,
        doc: &Document,
        mut pos: usize,
        params: UpdateBufPositionParams,
    ) -> bool {
        let text = doc.text();

        if pos == self.text_pos || pos > text.len_chars() {
            return false;
        }

        let UpdateBufPositionParams {
            update_saved_column,
            line_keep,
            allow_on_newline,
        } = params;

        let mut line_idx = text.char_to_line(pos);

        if let Some(line_keep) = line_keep {
            if line_idx != self.line_idx {
                match line_keep {
                    LineKeep::Max => {
                        let next_line_idx =
                            (self.line_idx + 1).min(text.len_lines().saturating_sub(1));
                        let next_line_char = text.line_to_char(next_line_idx);
                        pos = next_line_char - 1;
                    }
                    LineKeep::Min => {
                        pos = self.line_char;
                    }
                }

                line_idx = self.line_idx;
            }
        }

        if line_idx != self.line_idx {
            self.line_char = text.line_to_char(line_idx);
        }

        let line = text.line(line_idx);

        if !allow_on_newline
            && line.len_chars() > 1
            && matches!(text.get_char(pos), None | Some('\n'))
        {
            pos -= 1;
        }

        self.line_idx = line_idx;
        self.text_pos = pos;

        if update_saved_column {
            let distance = self.text_pos - self.line_char;

            self.saved_column = nth_next_grapheme_boundary(line, 0, distance);
        }

        true
    }

    fn set_mode_impl(&mut self, mode: Mode) -> Result<()> {
        let mode_pos = self
            .avail_modes
            .iter()
            .position(|m| *m == mode)
            .with_context(|| format!("Buffer is not capable to enter {mode}"))?;

        self.current_mode = mode_pos;

        Ok(())
    }
}

#[derive(Debug)]
pub struct UpdateBufPositionParams {
    /// Update saved column
    pub update_saved_column: bool,
    /// Keep position in line bounds
    pub line_keep: Option<LineKeep>,
    /// Allow placing position on trailing \n character
    pub allow_on_newline: bool,
}

impl Default for UpdateBufPositionParams {
    fn default() -> Self {
        Self {
            update_saved_column: true,
            line_keep: None,
            allow_on_newline: false,
        }
    }
}

impl UpdateBufPositionParams {
    pub const fn inserting_text() -> Self {
        Self {
            update_saved_column: true,
            line_keep: None,
            allow_on_newline: true,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LineKeep {
    Max,
    Min,
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
        assert_eq!(buffer.text_pos, 0);
        assert_eq!(buffer.saved_column, 0);

        let buffer = Buffer::new_text(1, &document).unwrap();
        assert_eq!(buffer.text_pos, 1);
        assert_eq!(buffer.saved_column, 1);

        let buffer = Buffer::new_text(2, &document).unwrap();
        assert_eq!(buffer.text_pos, 2);
        assert_eq!(buffer.saved_column, 2);

        let buffer = Buffer::new_text(3, &document).unwrap();
        assert_eq!(buffer.text_pos, 3);
        assert_eq!(buffer.saved_column, 3);

        let buffer = Buffer::new_text(4, &document).unwrap();
        assert_eq!(buffer.text_pos, 3, "Shouldn't be placed on newline");
        assert_eq!(buffer.saved_column, 3);

        assert!(
            Buffer::new_text(5, &document).is_err(),
            "Created buffer with position set out of document bounds"
        );

        *document.text_mut() = Rope::from("kaka\nk");
        let buffer = Buffer::new_text(5, &document).unwrap();

        assert_eq!(buffer.text_pos, 5);
        assert_eq!(buffer.saved_column, 0);

        *document.text_mut() = Rope::from("kaka");
        let buffer = Buffer::new_text(3, &document).unwrap();

        assert_eq!(buffer.text_pos, 3);
        assert_eq!(buffer.saved_column, 3);
    }

    #[test]
    fn mode_switch() {
        let modes = [Mode::Normal, Mode::Insert];

        let document = Document::new_scratch();
        let mut buffer = Buffer::new(0, modes, &document, Mode::Normal, false).unwrap();
        assert!(matches!(buffer.mode(), &Mode::Normal));

        buffer.switch_mode(Mode::Insert);
        assert!(buffer.mode().is_insert());
    }
}
