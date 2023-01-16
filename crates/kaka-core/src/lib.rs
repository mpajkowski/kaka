#![warn(
    clippy::perf,
    clippy::semicolon_if_nothing_returned,
    clippy::missing_const_for_fn,
    clippy::use_self
)]

pub mod document;
pub mod graphemes;
pub mod history;
pub mod languages;
pub mod selection;
pub mod shapes;
pub mod span;
pub mod transaction;

// re-export ropey
pub use ropey;

pub type SmartString = smartstring::SmartString<smartstring::LazyCompact>;
