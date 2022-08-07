use std::{
    cmp::Reverse,
    collections::{hash_map::Entry, HashMap},
};

use anyhow::{Context, Result};
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

    pub fn keymap_for_mode(&self, mode: &Mode) -> Result<&Keymap> {
        let mode = mode.name();
        self.keymaps
            .get(mode)
            .with_context(|| format!("Keymap for mode {mode} not registered"))
    }
}

#[derive(Debug, Default)]
pub struct Keymap(
    // FIXME: probably BTreeMap should be used, unfortunately KeyEvent does not implement Ord.
    // Change to BTreeMap when own type for this purpose is implemented.
    HashMap<KeyEvent, KeymapTreeElement>,
);

impl Keymap {
    pub fn feed(&self, event: KeyEvent) -> Option<&KeymapTreeElement> {
        self.0.get(&event)
    }

    pub fn with_mappings(
        mappings: impl IntoIterator<Item = (&'static str, Command)>,
    ) -> Result<Self> {
        let mut keymap = Self::default();

        let mut mappings = mappings
            .into_iter()
            .filter_map(|(m, c)| super::utils::parse_mapping(m).ok().map(|m| (m, c)))
            .collect::<Vec<_>>();

        // deepest first
        mappings.sort_unstable_by_key(|m| Reverse(m.0.len()));

        for (mapping, command) in mappings {
            let len = mapping.len();

            if len == 0 {
                continue;
            }

            let first = mapping[0];
            if let Entry::Vacant(e) = keymap.0.entry(first) {
                if len == 1 {
                    e.insert(KeymapTreeElement::Leaf(command));
                    // mapping loop is not going to be executed anyway
                    // to satisfy borrowck
                    continue;
                } else {
                    e.insert(KeymapTreeElement::Node(Self::default()));
                }
            }

            let mut node = keymap
                .0
                .get_mut(&first)
                .map(|elem| match elem {
                    KeymapTreeElement::Node(ref mut n) => n,
                    // invariant - nodes are always before leaves
                    KeymapTreeElement::Leaf(_) => unreachable!(),
                })
                .unwrap();

            for (idx, keycode) in mapping.into_iter().enumerate().skip(1) {
                if let Entry::Vacant(e) = node.0.entry(keycode) {
                    if idx < len - 1 {
                        e.insert(KeymapTreeElement::Node(Self::default()));
                    } else {
                        e.insert(KeymapTreeElement::Leaf(command));
                        // end of loop, satisfy borrowck
                        break;
                    }
                }

                node = node
                    .0
                    .get_mut(&keycode)
                    .map(|elem| match elem {
                        KeymapTreeElement::Node(ref mut n) => n,
                        KeymapTreeElement::Leaf(_) => unreachable!(),
                    })
                    .unwrap();
            }
        }

        Ok(keymap)
    }

    /// some input paths used for tests
    pub fn xd() -> Self {
        let mappings = [
            ("q", command!(close)),
            ("x", command!(print_a)),
            ("i", command!(enter_insert_mode)),
            ("gac", command!(print_a)),
            ("gz", command!(print_a)),
            ("<C-a>x", command!(print_a)),
            ("<C-a><C-b>d", command!(print_a)),
        ];

        Self::with_mappings(mappings).expect("covered in unit tests :)")
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
        println!("Keymap {keymap:#?}");
    }
}
