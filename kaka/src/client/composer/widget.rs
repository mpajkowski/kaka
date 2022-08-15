use std::any::Any;

use crossterm::event::Event;
use kaka_core::shapes::Rect;

use crate::client::surface::Surface;

use super::{Context, EventOutcome};

pub trait Widget: Any {
    fn draw(&self, area: Rect, surface: &mut Surface, ctx: &mut Context<'_>);

    fn should_update(&self) -> bool {
        true
    }

    fn handle_event(&mut self, _: &Event, _: &mut Context) -> EventOutcome {
        EventOutcome::ignored()
    }
}
