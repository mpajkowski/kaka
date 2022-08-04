use std::collections::HashMap;

use anyhow::Context;
use cascade::cascade;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{command::*, Mode};
use crate::command;

#[derive(Debug, Default)]
pub struct Keymaps {
    keymaps: HashMap<String, Keymap>,
}

impl Keymaps {
    pub fn register_keymap_for_mode(&mut self, mode: &Mode, keymap: Keymap) -> Option<Keymap> {
        self.keymaps.insert(mode.name().to_string(), keymap)
    }

    pub fn keymap_for_mode(&self, mode: &Mode) -> anyhow::Result<&Keymap> {
        let mode = mode.name();
        self.keymaps
            .get(mode)
            .with_context(|| format!("Keymap for mode {mode} not registered"))
    }
}

#[derive(Debug)]
pub struct Keymap(
    // FIXME: probably BTreeMap should be used, unfortunately KeyEvent does not implement Ord.
    // Change to BTreeMap when own type for this purpose is implemented.
    HashMap<KeyEvent, KeymapTreeElement>,
);

impl Keymap {
    pub fn feed(&self, event: KeyEvent) -> Option<&KeymapTreeElement> {
        self.0.get(&event)
    }

    pub fn _register_simple_mapping(
        &mut self,
        mapping: &[u8],
        _command_fn: CommandCallback,
    ) -> &mut Self {
        for ch in mapping.iter() {
            debug_assert!(
                ch.is_ascii_alphanumeric() || ch.is_ascii_whitespace() || ch.is_ascii_punctuation()
            );
        }
        todo!("Implement vim-like mapping parser")
    }

    /// used for tests
    ///
    /// defined input paths:
    ///
    /// x
    /// g - a - c
    /// g - z
    pub fn xd() -> Self {
        Self(cascade! {
            HashMap::new();
            ..insert(k('q'), KeymapTreeElement::Leaf(command!(close)));
            ..insert(k('x'), KeymapTreeElement::Leaf(command!(dummy)));
            ..insert(k('i'), KeymapTreeElement::Leaf(command!(enter_insert_mode)));
            ..insert(k('g'), KeymapTreeElement::Node(Self(cascade! {
                HashMap::new();
                ..insert(k('a'), KeymapTreeElement::Node(Self(cascade! {
                    HashMap::new();
                    ..insert(k('c'), KeymapTreeElement::Leaf(command!(dummy)));
                })));
                ..insert(k('z'), KeymapTreeElement::Leaf(command!(dummy)));
            })));
        })
    }

    pub fn insert_mode() -> Self {
        let esc = KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
        };
        let mut map = HashMap::new();
        map.insert(esc, KeymapTreeElement::Leaf(command!(enter_xd_mode)));
        Self(map)
    }
}

const fn k(c: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(c),
        modifiers: KeyModifiers::empty(),
    }
}

#[derive(Debug)]
pub enum KeymapTreeElement {
    Leaf(Command),
    Node(Keymap),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_keymap() {
        let keymap = Keymap::xd();
        println!("Keymap {keymap:?}");
    }
}
