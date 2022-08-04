mod app;
mod editor;
mod client;
mod jobs;

use std::io::{stdout, Stdout};

use app::App;
use crossterm::event::EventStream;

pub type Client = client::Client<client::CrosstermCanvas<Stdout>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let crossterm_canvas = client::CrosstermCanvas::new(stdout(), true)?;
    let client = Client::new(crossterm_canvas);

    let mut app = App::new(client);
    app.run(&mut EventStream::new()).await?;

    Ok(())
}
