use std::{num::NonZeroUsize, time::Instant};

use ropey::Rope;

#[derive(Debug)]
pub enum Change {
    /// Replace char
    Replace { with: char },

    /// Move to position
    MoveTo { pos: usize },

    /// Move to position
    MoveBy { offset: isize },

    /// Insert string
    Insert { content: String },

    /// Delete
    Delete { len: NonZeroUsize },
}

#[derive(Debug)]
pub struct Transaction {
    _time: Instant,
    _len: usize,
    len_new: usize,
    head: usize,
    changes: Vec<Change>,
}

impl Transaction {
    pub fn begin(rope: &Rope, pos: usize) -> Self {
        let len = rope.len_chars();

        Self {
            _time: Instant::now(),
            changes: Vec::default(),
            _len: len,
            len_new: len,
            head: pos,
        }
    }

    pub fn replace(&mut self, ch: char) -> &mut Self {
        self.changes.push(Change::Replace { with: ch });
        self
    }

    pub fn insert(&mut self, string: String) -> &mut Self {
        let string_len = string.len();
        self.changes.push(Change::Insert { content: string });
        self.len_new += string_len;
        self
    }

    pub fn move_to(&mut self, pos: usize) {
        self.changes.push(Change::MoveTo { pos });
    }

    pub fn move_by(&mut self, offset: isize) {
        self.changes.push(Change::MoveBy { offset });
    }

    pub fn delete_one(&mut self) -> &mut Self {
        self.delete(unsafe { NonZeroUsize::new_unchecked(1) })
    }

    pub fn delete(&mut self, len: NonZeroUsize) -> &mut Self {
        self.changes.push(Change::Delete { len });

        self.len_new -= 1;
        self
    }

    pub fn commit(&self, rope: &mut Rope) {
        let mut pos = self.head;
        for change in self.changes.iter() {
            match change {
                Change::Replace { with } => {
                    let range = pos..pos + 1;
                    rope.remove(range);
                    rope.insert_char(pos, *with);
                }
                Change::MoveTo { pos: new_pos } => {
                    pos = *new_pos;
                }
                Change::MoveBy { offset } => {
                    pos = ((pos as isize) + offset) as usize;
                }
                Change::Insert { content } => rope.insert(pos, content),
                Change::Delete { len } => {
                    let range = pos..pos + usize::from(*len);
                    rope.remove(range);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn replace() {
        let mut text = Rope::from("hello tx");
        Transaction::begin(&text, 0).replace('a').commit(&mut text);

        assert_eq!(text, "aello tx");
    }

    #[test]
    fn delete() {
        let mut text = Rope::from("hello tx");
        Transaction::begin(&text, 0)
            .delete(NonZeroUsize::new(2).unwrap())
            .commit(&mut text);

        Transaction::begin(&text, 1).delete_one().commit(&mut text);

        assert_eq!(text, "lo tx");
    }
}
