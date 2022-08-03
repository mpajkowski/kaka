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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentId(NonZeroUsize);

impl DocumentId {
    pub fn inner(self) -> usize {
        self.0.get()
    }

    pub fn next() -> Self {
        static IDS: AtomicUsize = AtomicUsize::new(1);
        Self(
            NonZeroUsize::new(IDS.fetch_add(1, Ordering::SeqCst))
                .expect("DocumentId counter is messed"),
        )
    }
}

pub struct Document {
    id: DocumentId,
    text: Rope,
    fs_metadata: Option<FilesystemMetadata>,
}

impl Document {
    #[must_use]
    pub fn new_scratch() -> Self {
        Self {
            id: DocumentId::next(),
            text: Rope::new(),
            fs_metadata: None,
        }
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
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
        })
    }

    pub fn is_scratch(&self) -> bool {
        self.fs_metadata.is_none()
    }

    pub fn text(&self) -> &Rope {
        &self.text
    }

    pub fn text_mut(&mut self) -> &mut Rope {
        &mut self.text
    }

    pub fn path(&self) -> Option<&Path> {
        self.fs_metadata.as_ref().map(|m| m.path.as_ref())
    }

    pub fn id(&self) -> DocumentId {
        self.id
    }
}

#[allow(unused)]
pub struct FilesystemMetadata {
    path: PathBuf,
    writable: bool,
}
