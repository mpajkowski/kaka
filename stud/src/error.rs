use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error occurred: {0}")]
    IOError(#[from] io::Error),
}
