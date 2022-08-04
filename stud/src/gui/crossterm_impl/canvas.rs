use std::io::{self, stdout, Write};

use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::{
        Attribute as CAttribute, Color as CColor, Print, SetAttribute, SetBackgroundColor,
        SetForegroundColor,
    },
    terminal::{Clear, ClearType, LeaveAlternateScreen},
    ExecutableCommand,
};

use stud_core::shapes::{Point, Rect};

use crate::gui::{Canvas, Cell, Color, Modifier};

use super::RawTerminalGuard;

pub struct CrosstermCanvas<T> {
    writer: T,
    rect: Rect,
    _raw_terminal_guard: Option<RawTerminalGuard>,
}

impl<T: Write> CrosstermCanvas<T> {
    pub fn new(writer: T, setup_environment: bool) -> Result<Self> {
        let (width, height) = crossterm::terminal::size()?;
        println!("size: {:?}", (width, height));
        let start_point = Point::new(0, 0);

        if setup_environment {
            Self::setup_panic();
        }

        let mut this = Self {
            writer,
            rect: Rect::new(start_point.x, start_point.y, width, height),
            _raw_terminal_guard: setup_environment.then(RawTerminalGuard::init).transpose()?,
        };

        this.set_cursor(start_point)?;
        this.clear()?;
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
        let mut prev_point: Option<Point> = None;
        let mut fg = Color::Reset;
        let mut bg = Color::Reset;
        let mut modifier = Modifier::empty();

        for (point, cell) in cells {
            if !matches!(prev_point, Some(p) if point.x == p.x +1 && point.y == p.y) {
                queue!(self.writer, MoveTo(point.x, point.y))?;
                //println!("move to {point:?}");
            }
            prev_point = Some(point);

            if cell.modifier != modifier {
                let diff = ModifierDiff {
                    from: modifier,
                    to: cell.modifier,
                };
                diff.queue(&mut self.writer)?;
                modifier = cell.modifier;
            }

            if cell.fg != fg {
                queue!(self.writer, SetForegroundColor(CColor::from(cell.fg)))?;
                fg = cell.fg;
            }

            if cell.bg != bg {
                queue!(self.writer, SetBackgroundColor(CColor::from(cell.bg)))?;
                bg = cell.bg;
            }

            queue!(self.writer, Print(&cell.symbol))?;
        }

        queue!(
            self.writer,
            SetForegroundColor(CColor::Reset),
            SetBackgroundColor(CColor::Reset),
            SetAttribute(CAttribute::Reset)
        )?;

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

impl From<Color> for CColor {
    fn from(c: Color) -> Self {
        match c {
            Color::Reset => CColor::Reset,
            Color::Black => CColor::Black,
            Color::Red => CColor::Red,
            Color::Green => CColor::Green,
            Color::Yellow => CColor::Yellow,
            Color::Blue => CColor::Blue,
            Color::Magenta => CColor::Magenta,
            Color::Cyan => CColor::Cyan,
            Color::Gray => CColor::Grey,
            Color::DarkGray => CColor::DarkGrey,
            Color::LightRed => CColor::Red,
            Color::LightGreen => CColor::Green,
            Color::LightYellow => CColor::Yellow,
            Color::LightBlue => CColor::Blue,
            Color::LightMagenta => CColor::Magenta,
            Color::LightCyan => CColor::Cyan,
            Color::White => CColor::White,
            Color::Rgb(r, g, b) => CColor::Rgb { r, g, b },
            Color::Indexed(i) => CColor::AnsiValue(i),
        }
    }
}

#[derive(Debug)]
struct ModifierDiff {
    pub from: Modifier,
    pub to: Modifier,
}

impl ModifierDiff {
    fn queue<W>(&self, mut w: W) -> Result<()>
    where
        W: io::Write,
    {
        let removed = self.from - self.to;
        if removed.contains(Modifier::REVERSED) {
            queue!(w, SetAttribute(CAttribute::NoReverse))?;
        }
        if removed.contains(Modifier::BOLD) {
            queue!(w, SetAttribute(CAttribute::NormalIntensity))?;
            if self.to.contains(Modifier::DIM) {
                queue!(w, SetAttribute(CAttribute::Dim))?;
            }
        }
        if removed.contains(Modifier::ITALIC) {
            queue!(w, SetAttribute(CAttribute::NoItalic))?;
        }
        if removed.contains(Modifier::UNDERLINED) {
            queue!(w, SetAttribute(CAttribute::NoUnderline))?;
        }
        if removed.contains(Modifier::DIM) {
            queue!(w, SetAttribute(CAttribute::NormalIntensity))?;
        }
        if removed.contains(Modifier::CROSSED_OUT) {
            queue!(w, SetAttribute(CAttribute::NotCrossedOut))?;
        }
        if removed.contains(Modifier::SLOW_BLINK) || removed.contains(Modifier::RAPID_BLINK) {
            queue!(w, SetAttribute(CAttribute::NoBlink))?;
        }

        let added = self.to - self.from;
        if added.contains(Modifier::REVERSED) {
            queue!(w, SetAttribute(CAttribute::Reverse))?;
        }
        if added.contains(Modifier::BOLD) {
            queue!(w, SetAttribute(CAttribute::Bold))?;
        }
        if added.contains(Modifier::ITALIC) {
            queue!(w, SetAttribute(CAttribute::Italic))?;
        }
        if added.contains(Modifier::UNDERLINED) {
            queue!(w, SetAttribute(CAttribute::Underlined))?;
        }
        if added.contains(Modifier::DIM) {
            queue!(w, SetAttribute(CAttribute::Dim))?;
        }
        if added.contains(Modifier::CROSSED_OUT) {
            queue!(w, SetAttribute(CAttribute::CrossedOut))?;
        }
        if added.contains(Modifier::SLOW_BLINK) {
            queue!(w, SetAttribute(CAttribute::SlowBlink))?;
        }
        if added.contains(Modifier::RAPID_BLINK) {
            queue!(w, SetAttribute(CAttribute::RapidBlink))?;
        }

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
