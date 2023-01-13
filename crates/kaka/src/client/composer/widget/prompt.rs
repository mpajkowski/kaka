use crossterm::event::Event;
use kaka_core::{
    shapes::{Point, Rect},
    SmartString,
};

use crate::client::{
    composer::layouter,
    style::{Color, Style},
    surface::Surface,
};

use super::{Context, EventOutcome, Widget};

pub struct PromptWidget {
    buffer: SmartString,
}

impl PromptWidget {
    pub fn new(char: char) -> Self {
        let mut buffer = SmartString::new_const();
        buffer.push(char);

        Self { buffer }
    }
}

impl Widget for PromptWidget {
    fn draw(&self, area: Rect, surface: &mut Surface, _ctx: &Context<'_>) {
        let line = area.y;
        let width = area.width;

        surface.set_stringn(
            Point::new(0, line),
            &self.buffer,
            width as usize,
            Style::default().fg(Color::Red),
        );
    }

    fn handle_event(
        &mut self,
        _area: Rect,
        event: &Event,
        _ctx: &mut Context,
    ) -> super::EventOutcome {
        if let Event::Key(k) = event {
            if let crossterm::event::KeyCode::Char(c) = k.code {
                self.buffer.push(c);
                EventOutcome::consumed()
            } else {
                EventOutcome::consumed().callback(|c| c.remove_widget::<Self>())
            }
        } else {
            EventOutcome::ignored()
        }
    }

    fn area(&self, viewport: Rect) -> Rect {
        layouter::prompt(viewport)
    }
}
