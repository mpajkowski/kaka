use std::fmt::Display;

use super::error::CanvasError;
use stud_core::shapes::{Point, Rect};

pub struct Cell {
    pub point: Point,
    pub symbol: String,
}

impl Cell {
    pub fn new(point: Point, symbol: impl Display) -> Self {
        Self {
            point,
            symbol: symbol.to_string(),
        }
    }
}

pub trait Canvas {
    fn draw<I: Iterator<Item = Cell>>(&mut self, contents: I) -> Result<(), CanvasError>;
    fn set_cursor(&mut self, point: Point) -> Result<(), CanvasError>;
    fn cursor(&mut self) -> Result<Point, CanvasError>;
    fn hide_cursor(&mut self) -> Result<(), CanvasError>;
    fn show_cursor(&mut self) -> Result<(), CanvasError>;
    fn clear(&mut self) -> Result<(), CanvasError>;
    fn shape(&self) -> Rect;
    fn flush(&mut self) -> Result<(), CanvasError>;
}
