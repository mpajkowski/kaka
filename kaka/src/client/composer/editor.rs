use crossterm::event::{Event, KeyCode, KeyEvent};
use kaka_core::shapes::{Point, Rect};

use super::{layouter, widget::Widget, Context, EventOutcome, EventResult};
use crate::{
    client::{
        style::{Color, Style},
        surface::Surface,
    },
    current, current_mut,
    editor::{self, insert_mode_on_key, Buffer, Command, KeymapTreeElement, Keymaps},
};

pub struct EditorWidget {
    buffered_keys: Vec<KeyEvent>,
    count: Option<usize>,
    insert_on: bool,
    cursor: Point,
}

impl Default for EditorWidget {
    fn default() -> Self {
        Self {
            buffered_keys: vec![],
            count: None,
            insert_on: false,
            cursor: Point::new(0, 0),
        }
    }
}

impl EditorWidget {
    fn reset(&mut self) {
        self.count = None;
        self.buffered_keys.clear();
    }

    fn update_count(&mut self, event: KeyEvent) {
        if self.insert_on {
            return;
        }

        let code = event.code;

        let count = match code {
            KeyCode::Char(c) if ('0'..='9').contains(&c) => c,
            _ => return,
        };

        if !self.buffered_keys.is_empty() {
            self.reset();
            return;
        }

        let count = (count as u8 - b'0') as usize;

        let new_count = self
            .count
            .unwrap_or(0)
            .checked_mul(10)
            .and_then(|c| c.checked_add(count));

        match new_count {
            Some(c) => self.count = Some(c),
            None => {
                self.reset();
            }
        }
    }

    fn find_command(
        &mut self,
        keymaps: &Keymaps,
        buffer: &Buffer,
        event: KeyEvent,
    ) -> Option<Command> {
        if self.insert_on {
            return None;
        }

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
                // keys are buffered...
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
                // ...here
                Some(KeymapTreeElement::Node(_)) => self.buffered_keys.push(event),
                None => self.reset(),
            },
            // ...and here
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
    fn draw(&self, area: Rect, surface: &mut Surface, ctx: &Context<'_>) {
        let (buf, doc) = current!(ctx.editor);

        let text = doc.text();

        let max_y = (area.height as usize).min(text.len_lines());
        let vscroll = buf.vscroll();

        let style = Style::default().fg(Color::Yellow).bg(Color::Black);

        for y in 0..max_y {
            let line = text.line(y + vscroll);

            let line_render = line.slice(0..(area.width as usize).min(line.len_chars()));

            surface.set_stringn(
                Point::new(area.x, y as u16),
                &line_render.to_string(),
                area.width as usize,
                style,
            );
        }
    }

    fn handle_event(
        &mut self,
        area: Rect,
        event: &Event,
        ctx: &mut Context,
    ) -> super::EventOutcome {
        let (buf, _) = current_mut!(ctx.editor);

        let key_event = match event {
            Event::Key(ev) => *ev,
            _ => return EventOutcome::ignored(),
        };

        self.update_count(key_event);
        let command = self.find_command(&ctx.editor.keymaps, buf, key_event);

        let is_insert = buf.mode().is_insert();

        let mut context = editor::CommandData {
            editor: ctx.editor,
            trigger: key_event,
            count: self.count,
            callback: None,
        };

        // TODO delegate to Mode?
        if let Some(command) = command {
            log::debug!("Invoking command: {}", command.describe());
            command.call(&mut context);
            self.reset();
        } else if is_insert {
            log::debug!("insert_mode_on_key");
            insert_mode_on_key(&mut context, key_event);
        }

        let callback = context.callback;

        let (buf, _) = current_mut!(ctx.editor);
        buf.update_vscroll(area.height as usize);

        self.cursor = ctx.editor.cursor(area);

        EventOutcome {
            callback,
            result: EventResult::Consumed,
        }
    }

    fn cursor(&self) -> Option<Point> {
        Some(self.cursor)
    }

    fn area(&self, viewport: Rect) -> Rect {
        layouter::editor(viewport)
    }
}

#[cfg(test)]
mod test {
    use crossterm::event::{KeyEventKind, KeyEventState, KeyModifiers};

    use super::*;

    #[test]
    fn count() {
        let mut event = KeyEvent {
            code: KeyCode::Char('2'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        let mut editor = EditorWidget::default();
        editor.update_count(event);
        assert_eq!(editor.count, Some(2));

        editor.update_count(event);
        assert_eq!(editor.count, Some(22));

        event.code = KeyCode::Char('3');
        editor.update_count(event);
        assert_eq!(editor.count, Some(223));
    }
}
