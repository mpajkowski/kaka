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
use kaka_core::languages::Languages;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let crossterm_canvas = client::CrosstermCanvas::new(stdout(), true)?;
    let client = Client::new(crossterm_canvas);

    let lang_loader = Languages::from_yaml("usr.share.kaka/languages.yaml")?;

    let mut app = App::new(client, lang_loader);
    app.run(std::env::args(), &mut EventStream::new()).await?;

    Ok(())
}
