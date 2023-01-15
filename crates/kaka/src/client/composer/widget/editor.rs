use std::sync::Arc;

use crossterm::event::{Event, KeyCode, KeyEvent};
use kaka_core::{
    shapes::{Point, Rect},
    span::{SpanIterator, SpanKind},
};

use super::{Context, Cursor, EventOutcome, Widget};
use crate::{
    client::{
        composer::{layouter, EventResult},
        style::{Color, CursorKind, Style},
        surface::Surface,
    },
    current, current_mut,
    editor::{self, insert_mode_on_key, Buffer, Command, KeymapTreeElement, Keymaps},
};

pub struct EditorWidget {
    buffered_keys: Vec<KeyEvent>,
    count: Option<usize>,
    insert_on: bool,
    cursor: Cursor,
}

impl Default for EditorWidget {
    fn default() -> Self {
        Self {
            buffered_keys: vec![],
            count: None,
            insert_on: false,
            cursor: Cursor(Point::new(0, 0), CursorKind::Block),
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
            KeyCode::Char(c) if c.is_ascii_digit() => c,
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
    ) -> Option<Arc<Command>> {
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
                call = Some(Arc::clone(command));
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

        if max_y == 0 {
            return;
        }

        let vscroll = buf.vscroll();

        let selection_range = buf.selection().map(|s| s.range());

        let style = Style::default().fg(Color::Yellow).bg(Color::Black);

        for y in 0..max_y {
            let line_idx = y + vscroll;
            let line = text.line(line_idx);
            let line_char = text.line_to_char(line_idx);
            let max_len = (area.width as usize).min(line.len_chars());

            let selection_range = selection_range.and_then(|(start, end)| {
                let overlaps = start <= line_char + max_len && line_char <= end;

                let start_in_line = start.saturating_sub(line_char).min(max_len);
                let end_in_line = end.saturating_sub(line_char).min(max_len);

                (start_in_line != end_in_line || overlaps).then_some((start_in_line, end_in_line))
            });

            SpanIterator::new(line, selection_range).for_each(|span| {
                let style = if span.kind.contains(SpanKind::SELECTION) {
                    style.bg(Color::Gray)
                } else {
                    style
                };

                let range = span.range;

                surface.set_stringn(
                    Point::new(area.x + range.start as u16, y as u16),
                    &line.slice(range).to_string(),
                    max_len,
                    style,
                );
            });
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut Context) -> super::EventOutcome {
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
            count: self.count,
            callback: None,
        };

        if let Some(command) = command {
            command.call(&mut context);
            self.reset();
        } else if is_insert {
            insert_mode_on_key(&mut context, key_event);
        }

        let callback = context.callback;

        EventOutcome {
            callback,
            result: EventResult::Consumed,
        }
    }

    fn cursor(&self) -> Option<Cursor> {
        Some(self.cursor)
    }

    fn area(&self, viewport: Rect) -> Rect {
        layouter::editor(viewport)
    }

    fn update_state(&mut self, area: Rect, ctx: &mut Context) {
        self.cursor = ctx.editor.cursor(area);

        let (buf, _) = current_mut!(ctx.editor);
        buf.update_vscroll(area.height as _);
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
