mod buffer;
mod command;
mod keymap;
mod mode;
pub mod utils;

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

pub use buffer::{Buffer, BufferId};
use crossterm::event::KeyEvent;
use kaka_core::document::{Document, DocumentId};
use kaka_core::ropey::Rope;
pub use keymap::{Keymap, KeymapTreeElement};
pub use mode::Mode;

use crate::client::Redraw;

pub use self::command::{insert_mode_on_key, Command, CommandData};
pub use self::keymap::Keymaps;

/// Holds editor state
pub struct Editor {
    pub buffers: BTreeMap<BufferId, Buffer>,
    pub documents: HashMap<DocumentId, Document>,
    pub current: BufferId,
    pub buffered_keys: Vec<KeyEvent>,
    pub exit_code: Option<i32>,
    pub keymaps: Keymaps,
    logger: BufferId,
}

impl Editor {
    pub fn init() -> Self {
        let mut keymaps = Keymaps::default();
        keymaps.register_keymap_for_mode(&Mode::Xd, Keymap::xd());
        keymaps.register_keymap_for_mode(&Mode::Insert, Keymap::insert_mode());
        keymaps.register_keymap_for_mode(&Mode::Normal, Keymap::normal_mode());

        Self {
            buffers: BTreeMap::new(),
            documents: HashMap::new(),
            current: BufferId::MAX,
            logger: BufferId::MAX,
            buffered_keys: Vec::new(),
            exit_code: None,
            keymaps,
        }
    }

    pub fn open(&mut self, path: impl AsRef<Path>, set_current: bool) -> anyhow::Result<()> {
        let document = Document::from_path(path)?;
        let buffer = Buffer::new_text(0, &document)?;

        self.add_buffer_and_document(buffer, document, set_current);

        Ok(())
    }

    pub fn open_scratch(&mut self, set_current: bool) {
        let document = Document::new_scratch();
        let buffer = Buffer::new_text(0, &document).expect("Should not fail");

        self.add_buffer_and_document(buffer, document, set_current);
    }

    pub fn add_buffer_and_document(
        &mut self,
        buffer: Buffer,
        document: Document,
        set_current: bool,
    ) {
        let buffer_id = buffer.id();
        self.documents.insert(document.id(), document);
        self.buffers.insert(buffer_id, buffer);

        if set_current {
            self.current = buffer_id;
        }
    }

    pub const fn should_exit(&self) -> bool {
        self.exit_code.is_some()
    }

    pub fn set_logger(&mut self, id: BufferId) {
        self.logger = id;
    }

    pub fn on_log(&mut self, log: Rope) -> Redraw {
        if let Some(log_doc) = self
            .buffers
            .get(&self.logger)
            .and_then(|buf| self.documents.get_mut(&buf.document_id()))
        {
            log_doc.text_mut().append(log);
        }

        Redraw(self.current == self.logger)
    }
}
