use crossterm::event::{Event, KeyEvent};
use kaka_core::shapes::{Point, Rect};

use super::{widget::Widget, Context, EventResult};
use crate::{
    client::{surface::Surface, Color, Style},
    current, current_mut,
    editor::{self, insert_mode_on_key, Buffer, Command, KeymapTreeElement, Keymaps},
};

#[derive(Default)]
pub struct EditorWidget {
    buffered_keys: Vec<KeyEvent>,
}

impl EditorWidget {
    fn find_command(
        &mut self,
        keymaps: &Keymaps,
        buffer: &Buffer,
        event: KeyEvent,
    ) -> Option<Command> {
        let (chain, keymap_element) = {
            let keymap = keymaps.keymap_for_mode(buffer.mode()).unwrap();

            self.buffered_keys.first().map_or_else(
                || (false, keymap.feed(event)),
                |buf1| (true, keymap.feed(*buf1)),
            )
        };

        let mut keymap_element = match keymap_element {
            Some(ke) => ke,
            None => return None,
        };

        for buf_key in self.buffered_keys.iter().skip(1) {
            keymap_element = match keymap_element {
                KeymapTreeElement::Node(k) => k.feed(*buf_key).unwrap(),
                KeymapTreeElement::Leaf(_) => unreachable!(),
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
        // TODO scroll
        let (_, doc) = current!(ctx.editor);

        let text = doc.text();

        let max_y = (area.height() as usize).min(text.len_lines());
        let end = text.line_to_char(max_y);

        let text = text.slice(0..end);

        let style = Style::default().fg(Color::Yellow).bg(Color::Black);

        for y in 0..max_y {
            let line = text.line(y);

            let line_render = line.slice(0..(area.width() as usize).min(line.len_chars()));

            surface.set_stringn(
                Point::new(0, y as u16),
                &line_render.to_string(),
                area.width() as usize,
                style,
            );
        }
    }

    fn handle_event(&mut self, event: Event, ctx: &mut Context) -> super::EventResult {
        let (buf, _) = current_mut!(ctx.editor);

        let key_event = match event {
            Event::Key(ev) => ev,
            _ => return EventResult::Ignored,
        };

        let command = self.find_command(&ctx.editor.keymaps, buf, key_event);

        let is_insert = buf.mode().is_insert();

        let mut context = editor::CommandData::new(ctx.editor, key_event);

        // TODO delegate to Mode?
        if let Some(command) = command {
            command.call(&mut context);
        } else if is_insert {
            insert_mode_on_key(&mut context, key_event);
        }

        EventResult::Consumed
    }
}
