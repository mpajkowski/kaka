#![warn(
    clippy::perf,
    clippy::semicolon_if_nothing_returned,
    clippy::missing_const_for_fn,
    clippy::use_self
)]

mod document;
pub mod shapes;

// re-export ropey
pub use ropey;

pub use document::{Document, DocumentId};
