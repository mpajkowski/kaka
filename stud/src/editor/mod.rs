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

use crate::{gui::Composer, Gui};

use self::command::{dummy, Command};

pub struct Editor {
    pub(super) buffers: HashMap<BufferId, Buffer>,
    pub(super) documents: HashMap<DocumentId, Document>,
    pub(super) current: BufferId,
    pub(super) mode_registry: ModeRegistry,
    pub(super) buffered_keys: Vec<KeyEvent>,
}

impl Editor {
    pub fn new() -> Self {
        let mut mode_registry = ModeRegistry::default();
        mode_registry.register(Mode::new(
            "cluncky",
            Keymap::xd(Command::new("dummy", dummy)),
        ));
        let scratch_document = Document::new_scratch();

        let init_buffer = Buffer::new(
            mode_registry.mode_by_name("cluncky").unwrap(),
            &scratch_document,
        );
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
            buffered_keys: vec![],
        }
    }

    pub(super) fn on_key_event(&mut self, event: KeyEvent, composer: &mut Composer) {}
}
