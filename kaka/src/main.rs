mod app;
mod editor;
mod gui;
mod jobs;

use std::io::{stdout, Stdout};

use app::App;
use crossterm::event::EventStream;

pub type Gui = gui::Gui<gui::CrosstermCanvas<Stdout>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let crossterm_canvas = gui::CrosstermCanvas::new(stdout(), true)?;
    let gui = Gui::new(crossterm_canvas);

    let mut app = App::new(gui);
    app.run(&mut EventStream::new()).await?;

    Ok(())
}
