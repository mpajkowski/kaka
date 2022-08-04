mod buffer;
mod command;
mod keymap;
mod mode;

use std::collections::HashMap;

pub use buffer::{Buffer, BufferId};
use crossterm::event::KeyEvent;
use kaka_core::{Document, DocumentId};
pub use keymap::{Keymap, KeymapTreeElement};
pub use mode::Mode;

pub use self::command::Command;
pub use self::keymap::Keymaps;

/// Holds editor state
pub struct Editor {
    pub buffers: HashMap<BufferId, Buffer>,
    pub documents: HashMap<DocumentId, Document>,
    pub current: BufferId,
    pub buffered_keys: Vec<KeyEvent>,
    pub exit_code: Option<i32>,
    pub keymaps: Keymaps,
}

impl Editor {
    pub fn init() -> Self {
        let mut keymaps = Keymaps::default();
        keymaps.register_keymap_for_mode(&Mode::Xd, Keymap::xd());
        keymaps.register_keymap_for_mode(&Mode::Insert, Keymap::insert_mode());

        let scratch_document = Document::new_scratch();

        let init_buffer = Buffer::new_text_buffer(&scratch_document);
        let init_buffer_id = init_buffer.id();

        Self {
            buffers: {
                let mut buffers = HashMap::new();
                buffers.insert(init_buffer_id, init_buffer);
                buffers
            },
            documents: {
                let mut documents = HashMap::new();
                documents.insert(scratch_document.id(), scratch_document);
                documents
            },
            current: init_buffer_id,
            buffered_keys: Vec::new(),
            exit_code: None,
            keymaps,
        }
    }

    pub fn should_exit(&self) -> bool {
        self.exit_code.is_some()
    }
}
