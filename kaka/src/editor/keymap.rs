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
            ("q", command!(close)),
            ("x", command!(print_a)),
            ("i", command!(switch_to_insert_mode_before)),
            ("a", command!(switch_to_insert_mode_after)),
            ("gac", command!(print_a)),
            ("gz", command!(print_a)),
            ("<C-a>x", command!(print_a)),
            ("<C-a><C-b>d", command!(print_a)),
        ];

        Self::with_mappings(mappings)
    }

    pub fn insert_mode() -> Self {
        Self::with_mappings([("<ESC>", command!(switch_to_normal_mode))])
    }

    pub fn normal_mode() -> Self {
        let mappings = [
            // mode
            ("i", command!(switch_to_insert_mode_before)),
            ("a", command!(switch_to_insert_mode_after)),
            // movement
            ("h", command!(move_left)),
            ("j", command!(move_down)),
            ("k", command!(move_up)),
            ("l", command!(move_right)),
            ("gg", command!(goto_line_default_top)),
            ("dd", command!(delete_line)),
            ("G", command!(goto_line_default_bottom)),
            ("u", command!(undo)),
            ("<C-r>", command!(redo)),
            ("<Space>xd", command!(switch_to_xd_mode)),
            ("zs", command!(save)), // tmp
            ("ZZ", command!(close)),
            ("x", command!(remove_char)),
            (":", command!(command_mode)),
            // buffer
            ("<TAB>", command!(buffer_next)),
            ("<S-TAB>", command!(buffer_prev)),
            ("<C-b>c", command!(buffer_create)),
            ("<C-b>k", command!(buffer_kill)),
        ];

        Self::with_mappings(mappings)
    }

    pub fn with_mappings(
        mappings: impl IntoIterator<Item = (&'static str, Command)>,
    ) -> Self {
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
                    e.insert(KeymapTreeElement::Leaf(command));
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
                        e.insert(KeymapTreeElement::Leaf(command));
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

        keymap
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
