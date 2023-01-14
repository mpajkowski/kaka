use std::borrow::Cow;

use crossterm::event::{Event, KeyCode};
use kaka_core::{
    shapes::{Point, Rect},
    SmartString,
};
use unicode_width::UnicodeWidthStr;

use crate::client::{
    composer::{layouter, Cursor},
    style::{Color, CursorKind, Style},
    surface::Surface,
};

use super::{Context, EventOutcome, Widget};

pub type OnExecuteCallback = Box<dyn Fn(&PromptWidget, &mut Context)>;

pub struct PromptWidget {
    greeter: Cow<'static, str>,
    buffer: SmartString,
    on_execute: OnExecuteCallback,
    cursor: Cursor,
}

impl PromptWidget {
    pub fn new(
        greeter: impl Into<Cow<'static, str>>,
        on_execute: impl Fn(&Self, &mut Context) + 'static,
    ) -> Self {
        Self {
            greeter: greeter.into(),
            buffer: SmartString::new_const(),
            on_execute: Box::new(on_execute),
            cursor: Cursor(Point::new(0, 0), CursorKind::Line),
        }
    }

    pub fn text(&self) -> &str {
        &self.buffer
    }
}

impl Widget for PromptWidget {
    fn draw(&self, area: Rect, surface: &mut Surface, _ctx: &Context<'_>) {
        surface.set_stringn(
            Point::new(area.x, area.y),
            format!("{}{}", self.greeter, self.buffer),
            area.width as usize,
            Style::default().fg(Color::Red),
        );
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut Context) -> super::EventOutcome {
        let retain = EventOutcome::consumed();
        let remove = || EventOutcome::consumed().callback(|c| c.remove_widget::<Self>());

        if let Event::Key(k) = event {
            match k.code {
                KeyCode::Char(ch) => {
                    self.buffer.push(ch);
                    retain
                }
                KeyCode::Enter => {
                    (self.on_execute)(self, ctx);
                    remove()
                }
                KeyCode::Backspace => {
                    if self.buffer.pop().is_some() {
                        retain
                    } else {
                        remove()
                    }
                }
                KeyCode::Esc => remove(),
                _ => retain,
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
        let greeter_width = self.greeter.width();
        let buffer_wdith = self.buffer.width();
        let width = (greeter_width + buffer_wdith).min(area.width as usize) as u16;

        self.cursor = Cursor(Point::new(width + area.x, area.y), CursorKind::Line);
    }
}
