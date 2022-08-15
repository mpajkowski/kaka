mod editor;
mod prompt;
mod widget;

use std::{any::TypeId, fmt};

use crossterm::event::Event;
pub use editor::EditorWidget;
pub use prompt::PromptWidget;

use kaka_core::shapes::{Point, Rect};

use crate::{current, editor::Editor};

pub use self::widget::Widget;

use super::{canvas::Canvas, surface::Surface};

pub type Callback = Box<dyn FnOnce(&mut Composer)>;

pub struct Context<'a> {
    pub editor: &'a mut Editor,
}

pub struct Composer {
    widgets: Vec<(TypeId, Box<dyn Widget>)>,
    surfaces: Surfaces,
    cursor: Point,
}

pub struct EventOutcome {
    pub callback: Option<Callback>,
    pub result: EventResult,
}

impl fmt::Debug for EventOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventOutcome")
            .field("callback", &self.callback.as_ref().map(|_| "with callback"))
            .field("result", &self.result)
            .finish()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EventResult {
    Ignored,
    Consumed,
}

impl EventOutcome {
    pub fn ignored() -> Self {
        Self {
            callback: None,
            result: EventResult::Ignored,
        }
    }

    pub fn consumed() -> Self {
        Self {
            callback: None,
            result: EventResult::Consumed,
        }
    }

    pub fn callback(mut self, c: impl FnOnce(&mut Composer) + 'static) -> Self {
        self.callback = Some(Box::new(c));
        self
    }
}

impl Composer {
    pub fn new(rect: Rect) -> Self {
        let surfaces = Surfaces::new(rect);

        Self {
            surfaces,
            widgets: vec![],
            cursor: Point::new(0, 0),
        }
    }

    pub fn render<C: Canvas>(
        &mut self,
        canvas: &mut C,
        ctx: &mut Context<'_>,
    ) -> anyhow::Result<()> {
        let current_surface = self.surfaces.surface_mut();
        let area = current_surface.area;

        for (_, w) in &self.widgets {
            w.draw(area, current_surface, ctx);
        }

        self.surfaces.render(canvas, self.cursor)?;

        Ok(())
    }

    pub fn handle_event(&mut self, event: Event, ctx: &mut Context) -> bool {
        let resized = if let Event::Resize(x, y) = event {
            self.surfaces.resize(Rect::new(0, 0, x, y));
            true
        } else {
            false
        };

        let mut consumed = false;
        let mut callbacks = vec![];
        for (_, widget) in self.widgets.iter_mut().rev() {
            let EventOutcome { callback, result } = widget.handle_event(&event, ctx);
            callbacks.extend(callback);
            consumed = result == EventResult::Consumed;
            if consumed {
                break;
            }
        }

        for callback in callbacks {
            callback(self);
        }

        let surface = self.surfaces.surface();

        let (buf, doc) = current!(ctx.editor);

        let text = doc.text();

        let pos = buf.text_position;
        let cursor_y = text.char_to_line(pos) % surface.area.height() as usize;
        let cursor_x = (pos - text.line_to_char(cursor_y)) % surface.area.width() as usize;

        self.cursor = Point::new(cursor_x as u16, cursor_y as u16);

        consumed || resized
    }

    pub fn push_widget<W: Widget + 'static>(&mut self, widget: W) {
        self.widgets.push((TypeId::of::<W>(), Box::new(widget)));
    }

    pub fn remove_widget<W: Widget>(&mut self) {
        self.widgets
            .retain(|(type_id, _)| *type_id != TypeId::of::<W>());
    }
}

struct Surfaces {
    surfaces: [Surface; 2],
    current_surface: usize,
}

impl Surfaces {
    pub fn new(rect: Rect) -> Self {
        let surfaces = [Surface::empty(rect), Surface::empty(rect)];

        Self {
            surfaces,
            current_surface: 0,
        }
    }

    pub fn surface_mut(&mut self) -> &mut Surface {
        &mut self.surfaces[self.current_surface]
    }

    pub const fn surface(&self) -> &Surface {
        &self.surfaces[self.current_surface]
    }

    pub fn render<C: Canvas>(&mut self, canvas: &mut C, cursor: Point) -> anyhow::Result<()> {
        let current_surface = &self.surfaces[self.current_surface];
        let prev_surface = &self.surfaces[1 - self.current_surface];

        canvas.hide_cursor()?;
        let diff = prev_surface.diff(current_surface);
        canvas.draw(diff)?;
        canvas.move_cursor(cursor)?;
        canvas.show_cursor()?;
        canvas.flush()?;

        // swap surfaces
        self.surfaces[1 - self.current_surface].reset();
        self.current_surface = 1 - self.current_surface;

        Ok(())
    }

    pub fn resize(&mut self, rect: Rect) {
        self.surfaces[self.current_surface].resize(rect);
        self.surfaces[1 - self.current_surface].resize(rect);
    }
}
