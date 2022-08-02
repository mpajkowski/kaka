use std::io::{stdout, Write};

use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::Print,
    terminal::{Clear, ClearType, LeaveAlternateScreen},
    ExecutableCommand,
};

use stud_core::shapes::{Point, Rect};

use crate::gui::{Canvas, Cell};

use super::RawTerminalGuard;

pub struct CrosstermCanvas<T> {
    writer: T,
    cursor: Point,
    rect: Rect,
    _raw_terminal_guard: Option<RawTerminalGuard>,
}

impl<T: Write> CrosstermCanvas<T> {
    pub fn new(writer: T, setup_environment: bool) -> Result<Self> {
        let (width, height) = crossterm::terminal::size()?;
        let start_point = Point::new(0, 0);

        if setup_environment {
            Self::setup_panic();
        }

        let mut this = Self {
            writer,
            cursor: start_point,
            rect: Rect::new(start_point.x, start_point.y, width, height),
            _raw_terminal_guard: setup_environment.then(RawTerminalGuard::init).transpose()?,
        };

        this.set_cursor(start_point)?;
        this.flush()?;

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

impl<T: Write> Canvas for CrosstermCanvas<T> {
    fn clear(&mut self) -> Result<()> {
        execute!(self.writer, Clear(ClearType::All))?;
        Ok(())
    }

    fn cursor(&mut self) -> Result<Point> {
        let (x, y) = crossterm::cursor::position()?;
        Ok(Point::new(x, y))
    }

    fn draw<'a, I: Iterator<Item = (Point, &'a Cell)>>(&mut self, cells: I) -> Result<()> {
        let mut prev_point = None;

        for (point, cell) in cells {
            if prev_point != Some(Point::new(point.x + 1, point.y)) {
                queue!(self.writer, MoveTo(point.x, point.y))?;
                prev_point = Some(point);
            }

            queue!(self.writer, Print(&cell.symbol))?;
        }

        self.writer.flush()?;

        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<()> {
        execute!(self.writer, Hide)?;
        Ok(())
    }

    fn set_cursor(&mut self, point: Point) -> Result<()> {
        queue!(self.writer, MoveTo(point.x, point.y))?;
        Ok(())
    }

    fn shape(&self) -> Rect {
        self.rect
    }

    fn show_cursor(&mut self) -> Result<()> {
        execute!(self.writer, Show)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn canvas_write() {
        let cells = [
            (
                Point::new(0, 0),
                &Cell {
                    symbol: "a".to_string(),
                    ..Default::default()
                },
            ),
            (
                Point::new(0, 1),
                &Cell {
                    symbol: "b".to_string(),
                    ..Default::default()
                },
            ),
            (
                Point::new(0, 2),
                &Cell {
                    symbol: "c".to_string(),
                    ..Default::default()
                },
            ),
        ];

        let mut data = vec![] as Vec<u8>;
        let cursor = Cursor::new(&mut data);
        let mut canvas = CrosstermCanvas::new(cursor, false).unwrap();
        canvas.draw(cells.into_iter()).unwrap();
        drop(canvas);

        println!("Data: {data:?}");
    }
}
