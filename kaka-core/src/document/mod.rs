mod error;

pub use error::Error;
use unicode_width::UnicodeWidthChar;

use std::{
    fs::File,
    io::BufReader,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use ropey::Rope;

use crate::{history::History, transaction::Transaction};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentId(NonZeroUsize);

impl DocumentId {
    #[must_use]
    pub fn next() -> Self {
        static IDS: AtomicUsize = AtomicUsize::new(1);
        Self(
            NonZeroUsize::new(IDS.fetch_add(1, Ordering::SeqCst))
                .expect("DocumentId counter is messed"),
        )
    }
}

#[derive(Debug)]
pub struct Document {
    id: DocumentId,
    text: Rope,
    tx_context: Option<TransactionContext>,
    fs_metadata: Option<FilesystemMetadata>,
    history: History,
}

impl Document {
    #[must_use]
    pub fn new_scratch() -> Self {
        Self {
            id: DocumentId::next(),
            text: Rope::new(),
            tx_context: None,
            fs_metadata: None,
            history: History::default(),
        }
    }

    /// Creates document from provided path
    ///
    /// # Returns
    ///
    /// `Document` with contents loaded from filesystem
    ///
    /// # Errors
    ///
    /// `io::Error` - file not found | lack of permissions
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();

        let mut doc = Self::new_scratch();

        let mut doc_metadata = FilesystemMetadata {
            path: path.to_owned(),
            writable: true, // TODO check parent metadata?
        };

        if !path.exists() {
            doc.fs_metadata = Some(doc_metadata);
            return Ok(doc);
        }

        let metadata = path.metadata()?;
        if !metadata.is_file() {
            return Err(Error::NotAFile(path.into()));
        }

        doc_metadata.writable = !metadata.permissions().readonly();

        let file = File::open(path)?;
        let text = Rope::from_reader(BufReader::new(file))?;

        doc.text = text;
        doc.fs_metadata = Some(doc_metadata);

        Ok(doc)
    }

    pub const fn is_scratch(&self) -> bool {
        self.fs_metadata.is_none()
    }

    pub const fn text(&self) -> &Rope {
        &self.text
    }

    pub fn text_mut(&mut self) -> &mut Rope {
        &mut self.text
    }

    pub fn path(&self) -> Option<&Path> {
        self.fs_metadata.as_ref().map(|m| m.path.as_ref())
    }

    pub const fn id(&self) -> DocumentId {
        self.id
    }

    pub fn column(&self, line_idx: usize, char_idx: usize) -> usize {
        let line = self.text.line(line_idx);

        (0..char_idx)
            .map(|i| line.char(i).width().unwrap_or(1))
            .sum()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(metadata) = self.fs_metadata.as_ref() {
            if metadata.writable {
                self.text.write_to(File::create(&metadata.path)?)?;
            }
        }

        Ok(())
    }

    #[track_caller]
    pub fn with_transaction<F>(
        &mut self,
        attach: TransactionAttachPolicy,
        pos: usize,
        mut callback: F,
    ) where
        F: FnMut(&mut Self, &mut Transaction) -> TransactionLeave,
    {
        // validate attach policy requirements
        match attach {
            TransactionAttachPolicy::RequireTransactionAlive => {
                assert!(self.tx_context.is_some());
            }
            TransactionAttachPolicy::Disallow => assert!(self.tx_context.is_none()),
            TransactionAttachPolicy::Allow => {}
        };

        // restore or create transaction context
        let tx_context = self.tx_context.take().unwrap_or_else(|| {
            let tx = Transaction::new(&self.text, pos);

            TransactionContext {
                transaction: tx,
                saved_text: self.text.clone(),
                start_pos: pos,
            }
        });

        let TransactionContext {
            mut transaction,
            saved_text,
            start_pos,
        } = tx_context;

        match callback(self, &mut transaction) {
            TransactionLeave::Commit => {
                self.history.create_commit(&saved_text, transaction);
            }
            TransactionLeave::Keep => {
                self.tx_context = Some(TransactionContext {
                    transaction,
                    saved_text,
                    start_pos,
                });
            }
            TransactionLeave::Rollback => {
                self.text = saved_text;
            }
        }
    }

    pub fn undo(&mut self) -> Option<usize> {
        self.history.undo().map(|tx| tx.apply(&mut self.text))
    }

    pub fn redo(&mut self) -> Option<usize> {
        self.history.redo().map(|tx| tx.apply(&mut self.text))
    }
}

#[derive(Debug)]
pub struct FilesystemMetadata {
    path: PathBuf,
    writable: bool,
}

#[derive(Debug)]
pub enum TransactionAttachPolicy {
    /// Require alive transaction on attach. Suitable for actions
    /// that don't finish on one command dispatch cycle
    RequireTransactionAlive,

    /// Allow subscribing to alive transaction. Suitable for actions
    /// like LSP textEdit
    Allow,

    /// Disallow subscribing to alive transaction
    Disallow,
}

/// Descibes what to do with transaction on scope exit
#[derive(Debug)]
pub enum TransactionLeave {
    /// Keep current transaction
    Keep,

    /// Commit changes
    Commit,

    /// Rollback changes
    Rollback,
}

#[derive(Debug)]
struct TransactionContext {
    transaction: Transaction,
    saved_text: Rope,
    start_pos: usize,
}
