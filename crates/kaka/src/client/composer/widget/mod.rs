mod editor;
mod prompt;

pub use editor::EditorWidget;
pub use prompt::PromptWidget;

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

    fn handle_event(&mut self, _event: &Event, _context: &mut Context) -> EventOutcome {
        EventOutcome::ignored()
    }

    fn cursor(&self) -> Option<Cursor> {
        None
    }

    fn update_state(&mut self, _: Rect, _context: &mut Context) {}

    /// Probably not a good idea but ok for now
    fn area(&self, viewport: Rect) -> Rect;
}
