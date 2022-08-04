use super::Keymap;

use std::{borrow::Cow, collections::HashMap, sync::Arc};

#[derive(Debug)]
pub struct Mode {
    name: Cow<'static, str>,
    keymap: Keymap,
}

impl Mode {
    #[must_use]
    pub fn new(name: impl Into<Cow<'static, str>>, keymap: Keymap) -> Self {
        Self {
            name: name.into(),
            keymap,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn keymap(&self) -> &Keymap {
        &self.keymap
    }
}

#[derive(Debug, Default)]
pub struct Registry {
    modes: HashMap<String, Arc<Mode>>,
}

impl Registry {
    pub fn register(&mut self, mode: Mode) {
        let k = mode.name.to_string();

        self.modes.insert(k, Arc::new(mode));
    }

    pub fn mode_by_name(&self, k: &str) -> Option<Arc<Mode>> {
        self.modes.get(k).cloned()
    }
}
