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

    pub const fn transaction_active(&self) -> bool {
        self.tx_context.is_some()
    }

    pub fn open_transaction(&mut self, pos: usize) {
        if let Some(ctx) = self.tx_context.as_mut() {
            ctx.transaction.move_to(pos);
        } else {
            let saved_text = self.text.clone();
            self.tx_context = Some(TransactionContext {
                transaction: Transaction::new(&saved_text, pos),
                saved_text,
            });
        }
    }

    pub fn with_new_transaction<F>(&mut self, pos: usize, callback: F)
    where
        F: FnMut(&mut Self, &mut Transaction) -> TransactionLeave,
    {
        assert!(self.tx_context.is_none());

        let saved_text = self.text.clone();
        self.tx_context = Some(TransactionContext {
            transaction: Transaction::new(&saved_text, pos),
            saved_text,
        });

        self.with_transaction(callback);
    }

    #[track_caller]
    pub fn with_transaction<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut Self, &mut Transaction) -> TransactionLeave,
    {
        assert!(self.tx_context.is_some());

        // restore or create transaction context
        let tx_context = self.tx_context.take().unwrap();

        let TransactionContext {
            mut transaction,
            saved_text,
        } = tx_context;

        match callback(self, &mut transaction) {
            TransactionLeave::Commit => {
                self.history.create_commit(&saved_text, transaction);
            }
            TransactionLeave::Keep => {
                self.tx_context = Some(TransactionContext {
                    transaction,
                    saved_text,
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
}

pub trait AsRope {
    fn as_rope(&self) -> &Rope;
    fn as_rope_mut(&mut self) -> &mut Rope;
}

impl AsRope for Rope {
    fn as_rope(&self) -> &Rope {
        self
    }

    fn as_rope_mut(&mut self) -> &mut Rope {
        self
    }
}

impl AsRope for Document {
    fn as_rope(&self) -> &Rope {
        self.text()
    }

    fn as_rope_mut(&mut self) -> &mut Rope {
        self.text_mut()
    }
}
