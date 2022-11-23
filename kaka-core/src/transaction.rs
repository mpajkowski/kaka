use std::{borrow::Cow, cmp::Ordering};

use ropey::Rope;
use smartstring::LazyCompact;

pub type SmartString = smartstring::SmartString<LazyCompact>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// Move forward by offset
    MoveForward { count: usize },

    /// Move backward by offset
    MoveBackward { count: usize },

    /// Insert string
    Insert { content: SmartString },

    /// Delete
    Delete { len: usize },
}

#[derive(Debug)]
pub struct Transaction {
    len_before: usize,
    len_after: usize,
    changes: Vec<Change>,
}

impl Transaction {
    pub fn new(rope: &Rope) -> Self {
        let len = rope.len_chars();

        Self {
            changes: Vec::default(),
            len_before: len,
            len_after: len,
        }
    }

    pub fn replace(&mut self, ch: char) {
        self.delete_one();
        let mut buf = [0; 4];
        let string = ch.encode_utf8(&mut buf);
        self.insert(string);
    }

    pub fn insert_char(&mut self, c: char) {
        let mut buf = SmartString::new_const();
        buf.push(c);

        self.insert(buf);
    }

    pub fn insert(&mut self, string: impl Into<SmartString>) {
        let string = string.into();

        log::trace!("Transaction::insert: {string}");

        if string.is_empty() {
            return;
        }

        let string_len = string.len();

        match self.changes.as_mut_slice() {
            [.., Change::Insert { content }] => content.push_str(&string),
            _ => self.changes.push(Change::Insert { content: string }),
        }

        self.len_after += string_len;
        log::trace!("Transaction::insert: len_new={}", self.len_after);
    }

    pub fn move_forward_by(&mut self, count: usize) {
        let fcount = count;

        log::trace!("Transaction::move_forward_by({fcount})");

        if fcount == 0 {
            return;
        }

        use Change::*;
        match self.changes.as_mut_slice() {
            [.., MoveForward { count: pfcount }] => *pfcount += fcount,
            [.., MoveBackward { count: bcount }] => match (*bcount).cmp(&fcount) {
                Ordering::Greater => *bcount -= count,
                Ordering::Equal => {
                    self.changes.pop();
                }
                Ordering::Less => {
                    *self.changes.last_mut().unwrap() = MoveForward {
                        count: fcount - *bcount,
                    }
                }
            },
            _ => self.changes.push(MoveForward { count: fcount }),
        }
    }

    pub fn move_forward_by_one(&mut self) {
        self.move_forward_by(1);
    }

    pub fn move_backward_by(&mut self, count: usize) {
        let bcount = count;

        log::trace!("Transaction::move_backward_by({bcount})");

        if bcount == 0 {
            return;
        }

        use Change::*;
        match self.changes.as_mut_slice() {
            [.., MoveBackward { count: pbcount }] => *pbcount += count,
            [.., MoveForward { count: fcount }] => match (*fcount).cmp(&bcount) {
                Ordering::Greater => *fcount -= bcount,
                Ordering::Equal => {
                    self.changes.pop();
                }
                Ordering::Less => {
                    *self.changes.last_mut().unwrap() = MoveBackward {
                        count: bcount - *fcount,
                    }
                }
            },
            _ => self.changes.push(MoveBackward { count: bcount }),
        }
    }

    pub fn delete(&mut self, len: usize) {
        log::trace!("Transaction::delete({len})");

        if len == 0 {
            return;
        }

        match self.changes.as_mut_slice() {
            [.., Change::Delete { len: prev_len }] => *prev_len += len,
            _ => self.changes.push(Change::Delete { len }),
        };

        self.len_after -= len;
        log::trace!("Transaction::delete: len_new={}", self.len_after);
    }

    pub fn delete_one(&mut self) {
        self.delete(1);
    }

    pub fn apply(&self, rope: &mut Rope) -> usize {
        log::trace!("Transaction::commit()");

        let mut pos = 0;

        for change in self.changes.iter() {
            match change {
                Change::MoveBackward { count } => {
                    log::trace!("Transaction::commit() move backward count={count}");
                    pos -= *count;
                }
                Change::MoveForward { count } => {
                    log::trace!("Transaction::commit() move forward count={count}");
                    pos += *count;
                }
                Change::Insert { content } => {
                    log::trace!(
                        "Transaction::commit() insert pos={pos} len={}",
                        content.len()
                    );
                    rope.insert(pos, content);
                    pos += content.len();
                }
                Change::Delete { len } => {
                    log::trace!("Transaction::commit() delete len={len}");
                    let range = pos..pos + *len;
                    rope.remove(range);
                }
            }
        }

        log::info!("Transaction::commit(): exit");

        pos
    }

    pub fn undo(&self, original: &Rope) -> Self {
        log::trace!("Transaction::undo()");

        debug_assert_eq!(original.len_chars(), self.len_before);

        let mut changes = Self {
            len_before: self.len_after,
            len_after: self.len_after,
            changes: vec![],
        };

        let mut pos = 0;

        for change in &self.changes {
            match change {
                Change::MoveForward { count } => {
                    pos += count;
                    changes.move_forward_by(*count);
                }
                Change::MoveBackward { count } => {
                    pos -= count;
                    changes.move_backward_by(*count);
                }
                Change::Insert { content } => {
                    let len = content.chars().count();

                    changes.delete(len);
                }
                Change::Delete { len } => {
                    let text = Cow::from(original.slice(pos..pos + len));
                    changes.insert(text);
                    pos += len;
                }
            };
        }

        changes
    }

    pub fn changes_text(&self) -> bool {
        for change in self.changes.iter() {
            if matches!(change, Change::Insert { .. } | Change::Delete { .. }) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn move_reduction() {
        let mut tx = Transaction::new(&Rope::default());

        //start

        // .>
        tx.move_forward_by(1);
        assert_eq!(tx.changes, vec![Change::MoveForward { count: 1 }]);

        // .>>
        tx.move_forward_by(1);
        assert_eq!(tx.changes, vec![Change::MoveForward { count: 2 }]);

        // .
        tx.move_backward_by(2);
        assert_eq!(tx.changes, vec![]);

        // .>
        tx.move_forward_by(1);
        assert_eq!(tx.changes, vec![Change::MoveForward { count: 1 }]);

        // <.
        tx.move_backward_by(2);
        assert_eq!(tx.changes, vec![Change::MoveBackward { count: 1 }]);

        // .>>
        tx.move_forward_by(3);
        assert_eq!(tx.changes, vec![Change::MoveForward { count: 2 }]);
    }

    #[test]
    fn replace() {
        let mut text = Rope::from("hello tx");
        let mut tx = Transaction::new(&text);
        tx.replace('a');
        tx.apply(&mut text);

        assert_eq!(text, "aello tx");
    }

    #[test]
    fn delete() {
        let mut text = Rope::from("hello tx");
        let mut tx = Transaction::new(&text);
        tx.delete(2);
        tx.apply(&mut text);

        let mut tx = Transaction::new(&text);
        tx.move_forward_by_one();
        tx.delete_one();
        tx.apply(&mut text);

        assert_eq!(text, "lo tx");
    }

    #[test]
    fn undo() {
        let original_text = Rope::from("hello tx");
        let mut transformed_text = original_text.clone();

        let mut tx = Transaction::new(&original_text);
        tx.move_forward_by_one();
        tx.delete(3);
        tx.insert("xxxy");
        tx.apply(&mut transformed_text);

        assert_eq!(transformed_text, "hxxxyo tx");

        let inverse = tx.undo(&original_text);
        inverse.apply(&mut transformed_text);

        assert_eq!(original_text, transformed_text);
    }
}
