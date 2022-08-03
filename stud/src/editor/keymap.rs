use std::collections::HashMap;

use cascade::cascade;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::command::{Command, CommandCallback};
use crate::command;

use super::command::{close, dummy};

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

    pub fn register_simple_mapping(
        &mut self,
        mapping: &[u8],
        command_fn: CommandCallback,
    ) -> &mut Self {
        for ch in mapping.iter() {
            debug_assert!(
                ch.is_ascii_alphanumeric() || ch.is_ascii_whitespace() || ch.is_ascii_punctuation()
            )
        }
        self
    }

    /// used for tests
    ///
    /// defined input paths:
    ///
    /// x
    /// g - a - c
    /// g - z
    pub fn xd() -> Self {
        let k = |c: char| KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::empty(),
        };

        Self(cascade! {
            HashMap::new();
            ..insert(k('q'), KeymapTreeElement::Leaf(command!(close)));
            ..insert(k('x'), KeymapTreeElement::Leaf(command!(dummy)));
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
}

#[derive(Debug)]
pub enum KeymapTreeElement {
    Leaf(Command),
    Node(Keymap),
}

#[cfg(test)]
mod test {
    use crate::editor::command::dummy;

    use super::*;

    #[test]
    fn test_keymap() {
        let cmd = Command::new("dummy", dummy);
        let keymap = Keymap::xd();
        println!("Keymap {keymap:?}");
    }
}
