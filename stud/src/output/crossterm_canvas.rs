use std::io::Write;

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::Print,
    terminal::{Clear, ClearType},
};
use stud_core::shapes::{Point, Rect};

use super::{
    canvas::{Canvas, Cell},
    error::CanvasError,
};

pub struct CrosstermCanvas<T: Write> {
    writer: T,
    cursor: Point,
    rect: Rect,
}

impl<T: Write> CrosstermCanvas<T> {
    pub fn new(writer: T) -> Result<Self, CanvasError> {
        let (width, height) = crossterm::terminal::size()?;
        let start_point = Point::new(0, 0);

        let mut this = Self {
            writer,
            cursor: start_point,
            rect: Rect::new(start_point.x, start_point.y, width, height),
        };

        this.set_cursor(start_point)?;

        Ok(this)
    }
}

impl<T: Write> Canvas for CrosstermCanvas<T> {
    fn clear(&mut self) -> Result<(), CanvasError> {
        execute!(self.writer, Clear(ClearType::All))?;
        Ok(())
    }

    fn cursor(&mut self) -> Result<Point, CanvasError> {
        let (x, y) = crossterm::cursor::position()?;
        Ok(Point::new(x, y))
    }

    fn draw<I: Iterator<Item = Cell>>(&mut self, cells: I) -> Result<(), CanvasError> {
        let mut prev_point = None;

        for cell in cells {
            let point = cell.point;

            if prev_point != Some(Point::new(point.x + 1, point.y)) {
                queue!(self.writer, MoveTo(point.x, point.y))?;
                prev_point = Some(point);
            }

            queue!(self.writer, Print(cell.symbol))?;
        }

        self.writer.flush()?;

        Ok(())
    }

    fn flush(&mut self) -> Result<(), CanvasError> {
        self.writer.flush()?;
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<(), CanvasError> {
        execute!(self.writer, Hide)?;
        Ok(())
    }

    fn set_cursor(&mut self, point: Point) -> Result<(), CanvasError> {
        queue!(self.writer, MoveTo(point.x, point.y))?;
        Ok(())
    }

    fn shape(&self) -> Rect {
        self.rect
    }

    fn show_cursor(&mut self) -> Result<(), CanvasError> {
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
            Cell::new(Point::new(0, 0), "a"),
            Cell::new(Point::new(0, 1), "b"),
            Cell::new(Point::new(0, 2), "b"),
        ];

        let mut data = vec![] as Vec<u8>;
        let cursor = Cursor::new(&mut data);
        let mut canvas = CrosstermCanvas::new(cursor).unwrap();
        canvas.draw(cells.into_iter()).unwrap();

        println!("Data: {data:?}");
    }
}
