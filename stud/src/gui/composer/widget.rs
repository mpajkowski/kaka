use anyhow::Result;
use stud_core::shapes::Rect;

use crate::gui::surface::Surface;

pub trait Widget {
    fn draw(&self, area: Rect, surface: &mut Surface) -> Result<()>;
}
