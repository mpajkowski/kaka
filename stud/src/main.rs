mod app;
mod editor;
mod error;
mod jobs;
mod output;

use app::App;
use crossterm::event::EventStream;
pub use error::Error;
use output::Output;

pub type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    let output = Output::init()?;

    let mut app = App::new(output);
    app.run(&mut EventStream::new()).await?;

    Ok(())
}
