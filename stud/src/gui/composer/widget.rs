use crossterm::event::Event;
use stud_core::shapes::Rect;

use crate::gui::surface::Surface;

use super::{Context, EventResult};

pub trait Widget {
    fn draw(&self, area: Rect, surface: &mut Surface, ctx: &mut Context<'_>);

    fn should_update(&self) -> bool {
        true
    }

    fn handle_event(&mut self, _: Event, _: &mut Context) -> EventResult {
        EventResult::ignored()
    }
}
