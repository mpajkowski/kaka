use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO Error occurred: {0}")]
    Io(#[from] io::Error),

    #[error("{} not found", .0.display())]
    FileNotFound(PathBuf),

    #[error("{} is not a file", .0.display())]
    NotAFile(PathBuf),
}
