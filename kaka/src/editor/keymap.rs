use std::{
    cmp::Reverse,
    collections::{hash_map::Entry, HashMap},
};

use anyhow::{Context, Result};
use crossterm::event::KeyEvent;

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

    /// some input paths used for tests
    pub fn xd() -> Self {
        let mappings = [
            ("q", close as CommandFn),
            ("x", print_a),
            ("i", switch_to_insert_mode_before),
            ("a", switch_to_insert_mode_after),
            ("gac", print_a),
            ("gz", print_a),
            ("<C-a>x", print_a),
            ("<C-a><C-b>d", print_a),
        ];

        Self::with_mappings(mappings).expect("covered in unit tests :)")
    }

    pub fn insert_mode() -> Self {
        Self::with_mappings([("<ESC>", switch_to_normal_mode as CommandFn)]).unwrap()
    }

    pub fn normal_mode() -> Self {
        let mappings = [
            // mode
            ("i", switch_to_insert_mode_before as CommandFn),
            ("a", switch_to_insert_mode_after),
            // movement
            ("h", move_left),
            ("j", move_down),
            ("k", move_up),
            ("l", move_right),
            ("gg", goto_line_default_top),
            ("G", goto_line_default_bottom),
            ("<Space>xd", switch_to_xd_mode),
            ("zs", save), // tmp
            ("ZZ", close),
            ("x", remove_char),
            (":", command_mode),
            // buffer
            ("<TAB>", buffer_next),
            ("<S-TAB>", buffer_prev),
            ("<C-b>c", buffer_create),
            ("<C-b>k", buffer_kill),
        ];

        Self::with_mappings(mappings).unwrap()
    }

    pub fn with_mappings(
        mappings: impl IntoIterator<Item = (&'static str, CommandFn)>,
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
                if len > 1 {
                    e.insert(KeymapTreeElement::Node(Self::default()));
                } else {
                    e.insert(KeymapTreeElement::Leaf(command!(command)));
                    continue;
                }
            }

            let mut node = keymap
                .0
                .get_mut(&first)
                .map(|elem| match elem {
                    KeymapTreeElement::Node(ref mut n) => n,
                    KeymapTreeElement::Leaf(_) => unreachable!(),
                })
                .unwrap();

            for (idx, keycode) in mapping.into_iter().enumerate().skip(1) {
                if let Entry::Vacant(e) = node.0.entry(keycode) {
                    if idx < len - 1 {
                        e.insert(KeymapTreeElement::Node(Self::default()));
                    } else {
                        e.insert(KeymapTreeElement::Leaf(command!(command)));
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
