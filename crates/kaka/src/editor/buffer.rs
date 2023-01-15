use anyhow::{ensure, Result};
use kaka_core::{
    document::{AsRope, Document, DocumentId},
    graphemes::nth_next_grapheme_boundary,
    selection::Selection,
};

use std::{
    cmp::Ordering,
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering as MemoryOrdering},
};

use super::{mode::ModeData, ModeKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BufferId(NonZeroUsize);

impl BufferId {
    pub const MAX: Self = Self(unsafe { NonZeroUsize::new_unchecked(usize::MAX) });

    pub fn next() -> Self {
        pub static IDS: AtomicUsize = AtomicUsize::new(1);

        let next = NonZeroUsize::new(IDS.fetch_add(1, MemoryOrdering::SeqCst))
            .expect("BufferId counter overflowed");

        Self(next)
    }
}

#[derive(Debug)]
pub struct Buffer {
    id: BufferId,
    document_id: DocumentId,
    avail_modes: Vec<ModeKind>,
    current_mode: ModeData,
    immortal: bool,
    saved_column: usize,
    text_pos: usize,
    line_idx: usize,
    line_char: usize,
    vscroll: usize,
}

impl Buffer {
    pub fn new_text(pos: usize, document: &Document) -> Result<Self> {
        Self::new(
            pos,
            [ModeKind::Normal, ModeKind::Insert, ModeKind::Visual], // todo bitflags?
            document,
            ModeKind::Normal,
            false,
        )
    }

    pub fn new_logging(document: &Document) -> Self {
        Self::new(
            0,
            [ModeKind::Normal, ModeKind::Visual],
            document,
            ModeKind::Normal,
            true,
        )
        .unwrap()
    }

    pub fn new(
        pos: usize,
        avail_modes: impl IntoIterator<Item = ModeKind>,
        document: &Document,
        start_mode: ModeKind,
        immortal: bool,
    ) -> Result<Self> {
        let text = document.text();

        ensure!(
            pos <= text.len_chars().saturating_sub(1),
            "Start position {pos} is out of bounds"
        );

        let avail_modes = avail_modes.into_iter().collect::<Vec<_>>();

        let mut this = Self {
            id: BufferId::next(),
            document_id: document.id(),
            avail_modes,
            current_mode: ModeData::Normal,
            text_pos: 0,
            saved_column: 0,
            line_idx: 0,
            line_char: 0,
            immortal,
            vscroll: 0,
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

    pub const fn mode(&self) -> ModeKind {
        self.current_mode.kind()
    }

    pub fn switch_mode(&mut self, mode: ModeKind) {
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

    pub const fn vscroll(&self) -> usize {
        self.vscroll
    }

    pub fn update_vscroll(&mut self, max: usize) {
        let lower_bound = self.vscroll;
        let upper_bound = self.vscroll + max - 1;

        if self.line_idx >= upper_bound {
            self.vscroll += self.line_idx - upper_bound;
        } else if self.line_idx < lower_bound {
            self.vscroll -= lower_bound - self.line_idx;
        }
    }

    pub const fn selection(&self) -> Option<&Selection> {
        if let ModeData::Visual(selection) = &self.current_mode {
            Some(selection)
        } else {
            None
        }
    }

    pub fn update_text_position(
        &mut self,
        rope: &impl AsRope,
        pos: usize,
        params: UpdateBufPositionParams,
    ) -> Option<usize> {
        let text = rope.as_rope();

        if pos > text.len_chars() {
            return None;
        }

        let UpdateBufPositionParams {
            update_saved_column,
            line_keep,
            allow_on_newline,
        } = params;

        let mut line_idx = text.char_to_line(pos);

        let mut new_pos = pos;

        if line_keep {
            match line_idx.cmp(&self.line_idx) {
                Ordering::Less => {
                    new_pos = self.line_char;
                }
                Ordering::Greater => {
                    let next_line_idx = (self.line_idx + 1).min(text.len_lines().saturating_sub(1));
                    let next_line_char = text.line_to_char(next_line_idx);
                    new_pos = next_line_char - 1;
                }
                Ordering::Equal => {}
            };

            line_idx = self.line_idx;

            debug_assert_eq!(line_idx, text.char_to_line(new_pos), "line changed");
        }

        if line_idx != self.line_idx {
            self.line_char = text.line_to_char(line_idx);
        }

        let line = text.line(line_idx);

        if !allow_on_newline
            && line.len_chars() > 1
            && matches!(text.get_char(new_pos), None | Some('\n'))
        {
            new_pos -= 1;
        }

        self.line_idx = line_idx;
        let old_pos = std::mem::replace(&mut self.text_pos, new_pos);

        if update_saved_column {
            let distance = self.text_pos - self.line_char;

            self.saved_column = nth_next_grapheme_boundary(line, 0, distance);
        }

        if old_pos != self.text_pos {
            self.current_mode.update(pos);
        }

        (self.text_pos != pos).then_some(new_pos)
    }

    fn set_mode_impl(&mut self, mode: ModeKind) -> Result<()> {
        anyhow::ensure!(
            self.avail_modes.contains(&mode),
            "Buffer is not capable to enter {mode}"
        );

        self.current_mode = ModeData::new(mode, self.text_pos);

        Ok(())
    }
}

#[derive(Debug)]
pub struct UpdateBufPositionParams {
    /// Update saved column
    pub update_saved_column: bool,
    /// Keep position in line bounds
    pub line_keep: bool,
    /// Allow placing position on trailing \n character
    pub allow_on_newline: bool,
}

impl Default for UpdateBufPositionParams {
    fn default() -> Self {
        Self {
            update_saved_column: true,
            line_keep: false,
            allow_on_newline: false,
        }
    }
}

impl UpdateBufPositionParams {
    pub const fn inserting_text() -> Self {
        Self {
            update_saved_column: true,
            line_keep: false,
            allow_on_newline: true,
        }
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
        let modes = [ModeKind::Normal, ModeKind::Insert];

        let document = Document::new_scratch();
        let mut buffer = Buffer::new(0, modes, &document, ModeKind::Normal, false).unwrap();
        assert!(matches!(buffer.mode(), ModeKind::Normal));

        buffer.switch_mode(ModeKind::Insert);
        assert!(buffer.mode().is_insert());
    }
}
