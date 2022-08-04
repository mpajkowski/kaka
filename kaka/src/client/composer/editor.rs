use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use kaka_core::shapes::Rect;

use super::{widget::Widget, Context, EventResult};
use crate::{
    editor::{Buffer, Command, KeymapTreeElement},
    client::{surface::Surface, Color},
};

#[derive(Default)]
pub struct EditorWidget {
    buffered_keys: Vec<KeyEvent>,
}

impl EditorWidget {
    fn find_command(&mut self, buffer: &Buffer, event: KeyEvent) -> Option<Command> {
        let (chain, keymap_element) = {
            let keymap = buffer.keymap();

            if let Some(buf1) = self.buffered_keys.first() {
                (true, keymap.feed(*buf1))
            } else {
                (false, keymap.feed(event))
            }
        };

        let mut keymap_element = match keymap_element {
            Some(ke) => ke,
            None => return None,
        };

        for buf_key in self.buffered_keys.iter().skip(1) {
            keymap_element = match keymap_element {
                KeymapTreeElement::Node(k) => k.feed(*buf_key).unwrap(),
                _ => unreachable!(),
            };
        }

        let mut call = None;
        match keymap_element {
            KeymapTreeElement::Node(n) if chain => match n.feed(event) {
                Some(KeymapTreeElement::Leaf(command)) => {
                    call = Some(command.clone());
                    self.buffered_keys.clear();
                }
                Some(KeymapTreeElement::Node(_)) => self.buffered_keys.push(event),
                None => self.buffered_keys.clear(),
            },
            KeymapTreeElement::Node(_) => self.buffered_keys.push(event),
            KeymapTreeElement::Leaf(command) => {
                call = Some(command.clone());
                self.buffered_keys.clear();
            }
        }
        call
    }
}

impl Widget for EditorWidget {
    fn draw(&self, area: Rect, surface: &mut Surface, ctx: &mut Context<'_>) {
        let (_, doc) = ctx.current_buffer_and_doc();

        let area = area.area() as usize;
        let text = doc.text();
        let count = area.min(text.len_chars());

        let text_slice = text.slice(..count);

        for (ch, cell) in text_slice.chars().zip(&mut surface.content[..count]) {
            cell.symbol = ch.to_string();
            cell.fg = Color::Yellow;
            cell.bg = Color::Black;
        }
    }

    fn handle_event(&mut self, event: Event, ctx: &mut Context) -> super::EventResult {
        let (buf, doc) = ctx.current_buffer_and_doc();

        let key_event = match event {
            Event::Key(ev) => ev,
            _ => return EventResult::ignored(),
        };

        if buf.mode().name() == "insert" {}

        let command = self.find_command(buf, key_event);

        if let Some(command) = command {
            command.call(ctx.editor)
        } else if buf.mode().name() == "insert" {
            if let KeyCode::Char(c) = key_event.code {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    doc.text_mut().append(c.to_uppercase().to_string().into())
                } else {
                    doc.text_mut().append(c.to_string().into());
                }
            }
        }

        EventResult::Consumed(None)
    }
}
