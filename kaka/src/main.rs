#![warn(
    clippy::perf,
    clippy::semicolon_if_nothing_returned,
    clippy::missing_const_for_fn,
    clippy::use_self
)]

mod app;
mod client;
mod editor;
pub mod logger;
mod macros;

use std::io::stdout;

use app::App;
use client::Client;
use crossterm::event::EventStream;

pub use client::Canvas;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let crossterm_canvas = client::CrosstermCanvas::new(stdout(), true)?;
    let client = Client::new(crossterm_canvas);

    let mut app = App::new(client);
    app.run(std::env::args(), &mut EventStream::new()).await?;

    Ok(())
}
