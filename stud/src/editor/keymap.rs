use std::collections::HashMap;

use cascade::cascade;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{command::Command, Editor};

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

    /// used for tests
    ///
    /// defined input paths:
    ///
    /// x
    /// g - a - c
    /// g - z
    pub fn xd(command_dummy: Command) -> Self {
        let k = |c: char| KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::empty(),
        };

        Self(cascade! {
            HashMap::new();
            ..insert(k('x'), KeymapTreeElement::Leaf(command_dummy.clone()));
            ..insert(k('g'), KeymapTreeElement::Node(Self(cascade! {
                HashMap::new();
                ..insert(k('a'), KeymapTreeElement::Node(Self(cascade! {
                    HashMap::new();
                    ..insert(k('c'), KeymapTreeElement::Leaf(command_dummy.clone()));
                })));
                ..insert(k('z'), KeymapTreeElement::Leaf(command_dummy));
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
        let keymap = Keymap::xd(cmd);
        println!("Keymap {keymap:?}");
    }
}
