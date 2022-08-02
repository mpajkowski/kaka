use thiserror::Error;

#[derive(Debug, Error)]
pub enum CanvasError {
    #[error("IO error occurred: {0}")]
    Io(#[from] std::io::Error),
}
