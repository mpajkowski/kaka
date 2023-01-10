use std::time::{Duration, SystemTime};

use ropey::Rope;

use crate::transaction::Transaction;

#[derive(Debug, Default)]
pub struct History {
    commits: Vec<Commit>,
    head: usize,
}

impl History {
    pub fn create_commit(&mut self, text: &Rope, tx: Transaction) {
        if !tx.changes_text() {
            return;
        }

        let commit = Commit::new(text, tx);

        while self.head < self.commits.len() {
            self.commits.pop();
        }

        self.commits.push(commit);
        self.head += 1;
    }

    /// move history by one
    pub fn undo(&mut self) -> Option<&Transaction> {
        let index = self.head.checked_sub(1)?;

        self.head -= 1;

        Some(&self.commits[index].inversion)
    }

    pub fn redo(&mut self) -> Option<&Transaction> {
        let head = self.head;

        if head < self.commits.len() {
            self.head += 1;
            Some(&self.commits[head].transaction)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Commit {
    transaction: Transaction,
    inversion: Transaction,
    timestamp: Duration,
}

impl Commit {
    pub fn new(text: &Rope, tx: Transaction) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time wents backward");

        Self::with_timestamp(text, tx, timestamp)
    }

    pub fn with_timestamp(text: &Rope, tx: Transaction, timestamp: Duration) -> Self {
        let inversion = tx.undo(text);

        let commit = Self {
            transaction: tx,
            inversion,
            timestamp,
        };

        log::debug!("Creating commit {commit:#?}");

        commit
    }

    pub const fn timestamp(&self) -> Duration {
        self.timestamp
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn history() -> History {
        let mut history = History::default();

        for _ in 0..10 {
            history
                .commits
                .push(Commit::new(&Rope::new(), Transaction::new(&Rope::new(), 0)));
        }

        history.head = 10;

        assert_eq!(history.commits.len(), 10);

        history
    }

    #[test]
    fn undo() {
        let mut history = history();
        history.undo();

        assert_eq!(history.head, 9);
    }

    #[test]
    fn redo() {
        let mut history = history();
        history.redo();

        assert_eq!(history.head, 10);
    }
}
