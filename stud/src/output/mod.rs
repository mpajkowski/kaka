use std::io::{stdout, Write};

use crossterm::{
    cursor::MoveTo,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand, QueueableCommand,
};
use stud_core::Vec2d;

use crate::Result;

pub struct Output {
    raw_terminal_guard: RawTerminalGuard,
    dimensions: Vec2d,
}

impl Output {
    pub fn init() -> Result<Self> {
        let dimensions = crossterm::terminal::size()?;

        let this = Self {
            raw_terminal_guard: RawTerminalGuard::init()?,
            dimensions: Vec2d::from(dimensions),
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
