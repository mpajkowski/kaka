mod error;

pub use error::Error;

use std::{
    fs::File,
    io::BufReader,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use ropey::Rope;

use crate::transaction::Transaction;

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
    transaction: Option<Transaction>,
    fs_metadata: Option<FilesystemMetadata>,
}

impl Document {
    #[must_use]
    pub fn new_scratch() -> Self {
        Self {
            id: DocumentId::next(),
            text: Rope::new(),
            fs_metadata: None,
            transaction: None,
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

        if !path.exists() {
            return Ok(Self {
                id: DocumentId::next(),
                text: Rope::new(),
                transaction: None,
                fs_metadata: Some(FilesystemMetadata {
                    path: path.to_owned(),
                    writable: true, // TODO check parent metadata?
                }),
            });
        }

        let metadata = path.metadata()?;

        if !metadata.is_file() {
            return Err(Error::NotAFile(path.into()));
        }

        let writable = !metadata.permissions().readonly();

        let file = File::open(path)?;

        let text = Rope::from_reader(BufReader::new(file))?;

        Ok(Self {
            id: DocumentId::next(),

            text,
            fs_metadata: Some(FilesystemMetadata {
                path: path.into(),
                writable,
            }),
            transaction: None,
        })
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

    pub fn transaction_mut(&mut self) -> Option<&mut Transaction> {
        self.transaction.as_mut()
    }

    pub const fn transaction(&self) -> Option<&Transaction> {
        self.transaction.as_ref()
    }

    pub fn begin_tx(&mut self, pos: usize) {
        self.transaction = Some(Transaction::begin(&self.text, pos));
    }

    pub fn commit_tx(&mut self) {
        let tx = self.transaction.take();
        if let Some(tx) = tx {
            tx.commit(&mut self.text);
        }
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(metadata) = self.fs_metadata.as_ref() {
            if metadata.writable {
                self.text.write_to(File::create(&metadata.path)?)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct FilesystemMetadata {
    path: PathBuf,
    writable: bool,
}
