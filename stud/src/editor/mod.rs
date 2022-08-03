mod buffer;
mod command;
mod keymap;
mod mode;

use std::collections::HashMap;

pub use buffer::{Buffer, BufferId};
use crossterm::event::KeyEvent;
pub use keymap::{Keymap, KeymapTreeElement};
pub use mode::{Mode, Registry as ModeRegistry};
use stud_core::{Document, DocumentId};

pub use self::command::Command;

/// Holds editor state
pub struct Editor {
    pub buffers: HashMap<BufferId, Buffer>,
    pub documents: HashMap<DocumentId, Document>,
    pub current: BufferId,
    pub mode_registry: ModeRegistry,
    pub buffered_keys: Vec<KeyEvent>,
    pub exit_code: Option<i32>,
}

impl Editor {
    pub fn init() -> Self {
        let mut mode_registry = ModeRegistry::default();
        mode_registry.register(Mode::new("xd", Keymap::xd()));
        mode_registry.register(Mode::new("insert", Keymap::insert_mode()));
        let scratch_document = Document::new_scratch();

        let init_buffer = Buffer::new(mode_registry.mode_by_name("xd").unwrap(), &scratch_document);
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
            mode_registry,
            buffered_keys: Vec::new(),
            exit_code: None,
        }
    }

    pub fn current_buffer_and_doc(&mut self) -> (&mut Buffer, &mut Document) {
        self.buffers
            .get_mut(&self.current)
            .map(|buf| {
                let doc_id = buf.document_id();
                let doc = self.documents.get_mut(&doc_id).unwrap();

                (buf, doc)
            })
            .unwrap()
    }

    pub fn should_exit(&self) -> bool {
        self.exit_code.is_some()
    }
}
