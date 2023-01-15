use std::{
    cmp::Reverse,
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use anyhow::{Context, Result};
use crossterm::event::KeyEvent;

use super::{command::*, ModeKind};
use registry::Registry as CommandRegistry;

#[derive(Debug, Default)]
pub struct Keymaps {
    keymaps: HashMap<String, Keymap>,
}

impl Keymaps {
    pub fn register_keymap_for_mode(&mut self, mode: &ModeKind, keymap: Keymap) -> Option<Keymap> {
        self.keymaps.insert(mode.name().to_string(), keymap)
    }

    pub fn keymap_for_mode(&self, mode: ModeKind) -> Result<&Keymap> {
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

    pub fn insert_mode(registry: &CommandRegistry) -> Self {
        let c = |name: &str| {
            registry
                .mappable_command_by_name(name)
                .expect("Failed to find command")
        };

        Self::with_mappings([("<ESC>", c("switch_to_normal_mode"))])
    }

    pub fn visual_mode(registry: &CommandRegistry) -> Self {
        let c = |name: &str| {
            registry
                .mappable_command_by_name(name)
                .expect("Failed to find command")
        };

        let mappings = [
            // mode_switch
            ("<ESC>", c("switch_to_normal_mode")),
            (":", c("command_mode")),
            // movement
            ("h", c("move_left")),
            ("j", c("move_down")),
            ("k", c("move_up")),
            ("l", c("move_right")),
            ("gg", c("goto_line_default_top")),
            ("G", c("goto_line_default_bottom")),
            // text_manipulation
            ("x", c("kill")),
        ];

        Self::with_mappings(mappings)
    }

    pub fn normal_mode(registry: &CommandRegistry) -> Self {
        let c = |name: &str| {
            registry
                .mappable_command_by_name(name)
                .expect("Failed to find command")
        };

        let mappings = [
            // buffer_mgmt
            ("<TAB>", c("buffer_next")),
            ("<S-TAB>", c("buffer_prev")),
            ("<C-b>c", c("buffer_create")),
            ("<C-b>k", c("buffer_kill")),
            ("zs", c("save")), // tmp
            ("ZZ", c("close")),
            // mode_switch
            ("i", c("switch_to_insert_mode_inplace")),
            ("I", c("switch_to_insert_mode_line_start")),
            ("a", c("switch_to_insert_mode_after")),
            ("A", c("switch_to_insert_mode_line_end")),
            ("v", c("switch_to_visual_mode")),
            // movement
            ("h", c("move_left")),
            ("j", c("move_down")),
            ("k", c("move_up")),
            ("l", c("move_right")),
            ("gg", c("goto_line_default_top")),
            ("G", c("goto_line_default_bottom")),
            // text_manipulation
            ("dd", c("kill_line")),
            ("x", c("kill")),
            (":", c("command_mode")),
            // history
            ("u", c("undo")),
            ("<C-r>", c("redo")),
        ];

        Self::with_mappings(mappings)
    }

    pub fn with_mappings(mappings: impl IntoIterator<Item = (&'static str, Arc<Command>)>) -> Self {
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
    Leaf(Arc<Command>),
    Node(Keymap),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_keymap() {
        let registry = CommandRegistry::populate();
        let keymap = Keymap::normal_mode(&registry);
        println!("Keymap {keymap:#?}");
    }
}
