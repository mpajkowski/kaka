use std::any::Any;

use crossterm::event::Event;
use kaka_core::shapes::Rect;

use crate::client::surface::Surface;

use super::{Context, Cursor, EventOutcome};

pub trait Widget: Any {
    fn draw(&self, area: Rect, surface: &mut Surface, ctx: &Context<'_>);

    fn should_update(&self) -> bool {
        true
    }

    fn handle_event(
        &mut self,
        _area: Rect,
        _event: &Event,
        _context: &mut Context,
    ) -> EventOutcome {
        EventOutcome::ignored()
    }

    fn cursor(&self) -> Option<Cursor> {
        None
    }

    /// Probably not a good idea but ok for now
    fn area(&self, viewport: Rect) -> Rect;
}
