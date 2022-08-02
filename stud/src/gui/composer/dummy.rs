use crate::gui::style::Color;
use anyhow::Result;
use stud_core::shapes::Rect;

use super::Surface;
use super::Widget;

pub struct DummyWidget;

impl Widget for DummyWidget {
    fn draw(&self, _: Rect, surface: &mut Surface) -> Result<()> {
        surface
            .content
            .iter_mut()
            .enumerate()
            .for_each(|(idx, cell)| {
                cell.fg = Color::Green;
                cell.bg = Color::DarkGray;
                cell.symbol = format!("{}", idx % 9)
            });

        Ok(())
    }
}
