#![warn(
    clippy::perf,
    clippy::semicolon_if_nothing_returned,
    clippy::missing_const_for_fn,
    clippy::use_self
)]

pub mod document;
pub mod shapes;
pub mod transaction;

// re-export ropey
pub use ropey;
