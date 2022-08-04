use anyhow::Result;

use kaka_core::shapes::{Point, Rect};

use super::Cell;

pub trait Canvas {
    fn draw<'a, I: Iterator<Item = (Point, &'a Cell)>>(&mut self, contents: I) -> Result<()>;
    fn set_cursor(&mut self, point: Point) -> Result<()>;
    fn cursor(&mut self) -> Result<Point>;
    fn hide_cursor(&mut self) -> Result<()>;
    fn show_cursor(&mut self) -> Result<()>;
    fn clear(&mut self) -> Result<()>;
    fn shape(&self) -> Rect;
    fn flush(&mut self) -> Result<()>;
}
