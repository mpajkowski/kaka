mod canvas;
mod crossterm_canvas;
mod error;

use std::{
    fmt::Display,
    io::{stdout, Write},
};

use crossterm::{
    cursor::MoveTo,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand, QueueableCommand,
};
use stud_core::shapes::Point;

use crate::Result;

pub struct Output {
    _raw_terminal_guard: RawTerminalGuard,
    dimensions: Point,
}

impl Output {
    pub fn init() -> Result<Self> {
        let dimensions = crossterm::terminal::size()?;

        let this = Self {
            _raw_terminal_guard: RawTerminalGuard::init()?,
            dimensions: Point::from(dimensions),
        };

        Self::setup_panic();

        let mut stdout = stdout();
        stdout.queue(Clear(ClearType::All))?.queue(MoveTo(0, 0))?;

        stdout.flush()?;

        Ok(this)
    }

    fn setup_panic() {
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let mut stdout = stdout();
            stdout.execute(LeaveAlternateScreen).ok();
            crossterm::terminal::disable_raw_mode().ok();

            hook(info)
        }));
    }

    pub fn dump(&mut self, contents: impl Display) -> Result<()> {
        let mut stdout = stdout();
        stdout.queue(Clear(ClearType::All))?;
        write!(stdout, "{contents}")?;
        stdout.flush()?;

        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        let mut stdout = stdout();
        stdout.queue(Clear(ClearType::All))?;
        stdout.flush()?;

        Ok(())
    }
}

struct RawTerminalGuard;

impl RawTerminalGuard {
    fn init() -> Result<Self> {
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = stdout();
        stdout.execute(EnterAlternateScreen)?;
        Ok(RawTerminalGuard)
    }
}

impl Drop for RawTerminalGuard {
    fn drop(&mut self) {
        let f = || {
            let mut stdout = stdout();
            stdout.execute(LeaveAlternateScreen)?;
            crossterm::terminal::disable_raw_mode()?;

            Ok::<_, std::io::Error>(())
        };

        f().expect("Failed to restore terminal state")
    }
}
