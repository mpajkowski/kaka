use std::io::stdout;

use anyhow::Result;
use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

pub struct RawTerminalGuard;

impl RawTerminalGuard {
    pub fn init() -> Result<Self> {
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = stdout();
        stdout.execute(EnterAlternateScreen)?;
        Ok(Self)
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

        f().expect("Failed to restore terminal state");
    }
}
