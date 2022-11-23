use crossterm::event::Event;
use kaka_core::shapes::{Point, Rect};

use crate::client::{
    style::{Color, Style},
    surface::Surface,
};

use super::{widget::Widget, Context, EventOutcome};

pub struct PromptWidget {
    char: char,
    line: String,
}

impl PromptWidget {
    pub const fn new(char: char) -> Self {
        Self {
            char,
            line: String::new(),
        }
    }
}

impl Widget for PromptWidget {
    fn draw(&self, area: Rect, surface: &mut Surface, _ctx: &mut Context<'_>) {
        let line = area.height().saturating_sub(1);
        let width = area.width();

        surface.set_stringn(
            Point::new(0, line),
            format!("{}{}", self.char, self.line),
            width as usize,
            Style::default().fg(Color::Red),
        );
    }

    fn handle_event(&mut self, event: &Event, _ctx: &mut Context) -> super::EventOutcome {
        if let Event::Key(k) = event {
            if let crossterm::event::KeyCode::Char(c) = k.code {
                self.line.push(c);
                EventOutcome::consumed()
            } else {
                EventOutcome::consumed().callback(|c| c.remove_widget::<Self>())
            }
        } else {
            EventOutcome::ignored()
        }
    }
}
