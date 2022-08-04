mod app;
mod client;
mod editor;
mod jobs;
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
    app.run(&mut EventStream::new()).await?;

    Ok(())
}
