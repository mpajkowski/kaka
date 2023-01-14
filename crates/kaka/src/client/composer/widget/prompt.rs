use crossterm::event::{Event, KeyCode};
use kaka_core::{
    shapes::{Point, Rect},
    SmartString,
};

use crate::client::{
    composer::{layouter, Cursor},
    style::{Color, CursorKind, Style},
    surface::Surface,
};

use super::{Context, EventOutcome, Widget};

pub type OnExecuteCallback = Box<dyn Fn(&PromptWidget, &mut Context)>;

pub struct PromptWidget {
    buffer: SmartString,
    on_execute: OnExecuteCallback,
    cursor: Cursor,
}

impl PromptWidget {
    pub fn new<F: Fn(&Self, &mut Context) + 'static>(char: char, on_execute: F) -> Self {
        let mut buffer = SmartString::new_const();
        buffer.push(char);

        Self {
            buffer,
            on_execute: Box::new(on_execute),
            cursor: Cursor(Point::new(0, 0), CursorKind::Line),
        }
    }

    pub fn text(&self) -> &str {
        &self.buffer[1..]
    }
}

impl Widget for PromptWidget {
    fn draw(&self, area: Rect, surface: &mut Surface, _ctx: &Context<'_>) {
        surface.set_stringn(
            Point::new(area.x, area.y),
            &self.buffer,
            area.width as usize,
            Style::default().fg(Color::Red),
        );
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut Context) -> super::EventOutcome {
        if let Event::Key(k) = event {
            match k.code {
                KeyCode::Char(ch) => {
                    self.buffer.push(ch);
                    EventOutcome::consumed()
                }
                KeyCode::Enter => {
                    (self.on_execute)(self, ctx);
                    EventOutcome::consumed().callback(|c| c.remove_widget::<Self>())
                }
                KeyCode::Esc => EventOutcome::consumed().callback(|c| c.remove_widget::<Self>()),
                _ => EventOutcome::consumed(),
            }
        } else {
            EventOutcome::ignored()
        }
    }

    fn area(&self, viewport: Rect) -> Rect {
        layouter::prompt(viewport)
    }

    fn cursor(&self) -> Option<Cursor> {
        Some(self.cursor)
    }

    fn update_state(&mut self, area: Rect, _context: &mut Context) {
        self.cursor = Cursor(
            Point::new(
                area.x + self.buffer.len().min(u16::MAX as usize) as u16,
                area.y,
            ),
            CursorKind::Line,
        );
    }

    fn should_update(&self) -> bool {
        true
    }
}
