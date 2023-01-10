mod editor;
mod layouter;
mod prompt;
mod widget;

use std::{any::TypeId, fmt};

use crossterm::event::Event;
pub use editor::EditorWidget;
pub use prompt::PromptWidget;

use kaka_core::shapes::{Point, Rect};

use crate::editor::Editor;

pub use self::widget::Widget;

use super::{canvas::Canvas, style::CursorKind, surface::Surface, Redraw};

pub type Callback = Box<dyn FnOnce(&mut Composer)>;

pub struct Context<'a> {
    pub editor: &'a mut Editor,
}

type Widgets = Vec<(TypeId, (Box<dyn Widget>, Rect))>;

pub struct Composer {
    widgets: Widgets,
    surfaces: Surfaces,
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
        }
    }

    pub fn render<C: Canvas>(
        &mut self,
        canvas: &mut C,
        ctx: &mut Context<'_>,
    ) -> anyhow::Result<()> {
        let current_surface = self.surfaces.surface_mut();

        for (_, (w, area)) in &self.widgets {
            w.draw(*area, current_surface, ctx);
        }

        let cursor = self.widgets.iter().rev().find_map(|(_, (w, _))| w.cursor());

        self.surfaces.render(canvas, cursor)?;

        Ok(())
    }

    pub fn handle_event(&mut self, event: Event, ctx: &mut Context) -> Redraw {
        let resized = if let Event::Resize(x, y) = event {
            let viewport = Rect::new(0, 0, x, y);

            self.surfaces.resize(viewport);

            for (_, (w, area)) in self.widgets.iter_mut() {
                *area = w.area(viewport);
            }

            true
        } else {
            false
        };

        let mut consumed = false;
        let mut callbacks = vec![];
        for (_, (widget, area)) in self.widgets.iter_mut().rev() {
            let EventOutcome { callback, result } = widget.handle_event(*area, &event, ctx);
            callbacks.extend(callback);
            consumed = result == EventResult::Consumed;
            if consumed {
                break;
            }
        }

        for callback in callbacks {
            callback(self);
        }

        Redraw(consumed || resized)
    }

    pub fn push_widget<W: Widget + 'static>(&mut self, widget: W) {
        let viewport = self.surfaces.surface().area;
        let area = widget.area(viewport);

        self.widgets
            .push((TypeId::of::<W>(), (Box::new(widget), area)));
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

    pub fn render<C: Canvas>(
        &mut self,
        canvas: &mut C,
        cursor: Option<Cursor>,
    ) -> anyhow::Result<()> {
        let current_surface = &self.surfaces[self.current_surface];
        let prev_surface = &self.surfaces[1 - self.current_surface];

        canvas.hide_cursor()?;
        let diff = prev_surface.diff(current_surface);
        canvas.draw(diff)?;

        if let Some(cursor) = cursor {
            canvas.move_cursor(cursor.0)?;
            canvas.set_cursor_kind(cursor.1)?;
            canvas.show_cursor()?;
        }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor(pub Point, pub CursorKind);
