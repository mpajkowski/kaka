use std::{borrow::Cow, cmp::Ordering, num::NonZeroUsize};

use ropey::Rope;
use smartstring::LazyCompact;

pub type SmartString = smartstring::SmartString<LazyCompact>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// Move forward by offset
    MoveForward(usize),

    /// Insert string
    Insert(SmartString),

    /// Delete
    Delete(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    repeat: NonZeroUsize,
    len_before: usize,
    len_after: usize,
    changesets: Vec<ChangeSet>,
}

impl Transaction {
    pub fn new(rope: &Rope, pos: usize) -> Self {
        let len = rope.len_chars();

        Self {
            repeat: NonZeroUsize::new(1).unwrap(),
            changesets: vec![ChangeSet::new(pos)],
            len_before: len,
            len_after: len,
        }
    }

    pub fn replace(&mut self, ch: char) {
        self.delete(1);
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

        if string.is_empty() {
            return;
        }

        let strlen = self.changeset_head().insert(string);

        self.len_after += strlen;
    }

    pub fn move_forward_by(&mut self, count: usize) {
        self.changeset_head().move_forward_by(count);
    }

    pub fn move_to(&mut self, pos: usize) {
        let end_pos = self.changeset_head().end_pos;

        match pos.cmp(&end_pos) {
            Ordering::Greater => self.move_forward_by(pos - end_pos),
            Ordering::Less => self.move_backward_by(end_pos - pos),
            Ordering::Equal => {}
        }
    }

    pub fn move_backward_by(&mut self, count: usize) {
        let head = self.changeset_head();

        if head.changes.is_empty() {
            head.start_pos -= count;
            head.end_pos -= count;
        } else {
            let new_start_pos = head.end_pos - count;
            self.changesets.push(ChangeSet::new(new_start_pos));
        }
    }

    pub fn delete(&mut self, len: usize) {
        self.changeset_head().delete(len);
    }

    pub fn set_repeat(&mut self, repeat: usize) {
        self.repeat = repeat.try_into().expect("repeat must be greater than one");
    }

    pub fn apply(&self, rope: &mut Rope) -> usize {
        self.apply_impl(rope, false)
    }

    pub fn apply_repeats(&self, rope: &mut Rope) -> usize {
        self.apply_impl(rope, true)
    }

    #[track_caller]
    fn apply_impl(&self, rope: &mut Rope, only_repeats: bool) -> usize {
        let mut pos = 0;
        let mut offset = None;
        let pos1 = self.changesets[0].start_pos;

        let repeat = self.repeat.get() - only_repeats as usize;

        for _ in 0..repeat {
            for change_set in self.changesets.iter() {
                pos = change_set.apply(offset.unwrap_or(0), rope);
            }

            if offset.is_none() {
                offset = Some(pos as isize - pos1 as isize);
            }
        }

        pos
    }

    #[track_caller]
    pub fn undo(&self, original: &Rope) -> Self {
        debug_assert_eq!(original.len_chars(), self.len_before);

        let mut revert = self
            .changesets
            .iter()
            .map(|c| c.undo(original))
            .collect::<Vec<_>>();

        revert.reverse();

        Self {
            repeat: self.repeat,
            len_before: self.len_after,
            len_after: self.len_after,
            changesets: revert,
        }
    }

    pub fn changes_text(&self) -> bool {
        self.changesets.iter().any(|ch| ch.changes_text())
    }

    fn changeset_head(&mut self) -> &mut ChangeSet {
        self.changesets
            .last_mut()
            .expect("At least one changeset in transaction is expected")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeSet {
    start_pos: usize,
    end_pos: usize,
    changes: Vec<Change>,
}

impl ChangeSet {
    pub const fn new(pos: usize) -> Self {
        Self {
            start_pos: pos,
            end_pos: pos,
            changes: vec![],
        }
    }

    fn insert(&mut self, string: SmartString) -> usize {
        use Change::*;

        let strlen = string.chars().count();

        if strlen == 0 {
            return 0;
        }

        match self.changes.as_mut_slice() {
            [.., Insert(content)] => {
                content.push_str(&string);
            }
            _ => self.changes.push(Insert(string)),
        }

        self.end_pos += strlen;
        strlen
    }

    fn move_forward_by(&mut self, count: usize) {
        use Change::*;

        match self.changes.last_mut() {
            Some(MoveForward(prev_count)) => *prev_count += count,
            _ => self.changes.push(MoveForward(count)),
        }

        self.end_pos += count;
    }

    fn delete(&mut self, len: usize) {
        use Change::*;

        if len == 0 {
            return;
        }

        match self.changes.last_mut() {
            Some(Delete(prev_len)) => *prev_len += len,
            _ => self.changes.push(Delete(len)),
        };
    }

    fn apply(&self, offset: isize, rope: &mut Rope) -> usize {
        let mut pos = (offset + self.start_pos as isize) as usize;

        for change in self.changes.iter() {
            match change {
                Change::MoveForward(count) => {
                    pos += *count;
                }
                Change::Insert(content) => {
                    rope.insert(pos, content);
                    pos += content.len();
                }
                Change::Delete(len) => {
                    let range = pos..pos + *len;
                    rope.remove(range);
                }
            }
        }

        pos
    }

    fn changes_text(&self) -> bool {
        self.changes
            .iter()
            .any(|change| matches!(change, Change::Insert(_) | Change::Delete(_)))
    }

    #[track_caller]
    pub fn undo(&self, original: &Rope) -> Self {
        use Change::*;

        let mut revert = Self {
            start_pos: self.start_pos,
            end_pos: self.start_pos, // we will reach there
            changes: vec![],
        };

        for change in self.changes.iter() {
            match change {
                MoveForward(count) => {
                    revert.move_forward_by(*count);
                }
                Insert(content) => {
                    let len = content.chars().count();

                    revert.delete(len);
                }
                Delete(len) => {
                    let pos = revert.end_pos;
                    let text = Cow::from(original.slice(pos..pos + len));
                    revert.insert(text.into());
                }
            };
        }

        revert
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn replace() {
        let mut text = Rope::from("hello tx");
        let mut tx = Transaction::new(&text, 0);
        tx.replace('a');
        tx.apply(&mut text);

        assert_eq!(text, "aello tx");
    }

    #[test]
    fn delete() {
        let mut text = Rope::from("hello tx");
        let mut tx = Transaction::new(&text, 0);
        tx.delete(2);
        tx.apply(&mut text);

        let mut tx = Transaction::new(&text, 0);
        tx.move_forward_by(1);
        tx.delete(1);
        tx.apply(&mut text);

        assert_eq!(text, "lo tx");
    }

    #[test]
    fn undo() {
        let original_text = Rope::from("hello tx");
        let mut transformed_text = original_text.clone();

        let mut tx = Transaction::new(&original_text, 0);
        tx.move_forward_by(1);
        tx.delete(3);
        tx.insert("xxxy");
        tx.apply(&mut transformed_text);

        assert_eq!(transformed_text, "hxxxyo tx");

        let inverse = tx.undo(&original_text);
        inverse.apply(&mut transformed_text);

        assert_eq!(original_text, transformed_text);
    }

    #[test]
    fn repeat() {
        let test = "test";
        let repeat = 1000;

        let mut text = Rope::default();
        let mut tx = Transaction::new(&text, 0);

        tx.insert(test);
        tx.set_repeat(repeat);
        let undo = tx.undo(&text);

        tx.apply(&mut text);

        let expected = [test]
            .iter()
            .cycle()
            .take(repeat)
            .map(|x| x.to_string())
            .collect::<String>();

        assert_eq!(text, expected);

        undo.apply(&mut text);

        assert_eq!(text, "");
    }
}
