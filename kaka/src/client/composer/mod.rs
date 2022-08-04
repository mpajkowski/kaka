mod editor;
mod widget;

use crossterm::event::Event;
pub use editor::EditorWidget;

use kaka_core::shapes::Rect;

use crate::{editor::Editor, jobs::Jobs};

use self::widget::Widget;

use super::{canvas::Canvas, surface::Surface};

pub type Callback = Box<dyn FnOnce(&mut Composer, &mut Editor)>;

pub struct Context<'a> {
    pub editor: &'a mut Editor,
    pub jobs: &'a mut Jobs,
}

pub struct Composer {
    widgets: Vec<Box<dyn Widget>>,
    surfaces: Surfaces,
}

pub enum EventResult {
    Ignored(Option<Callback>),
    Consumed(Option<Callback>),
}

impl EventResult {
    pub fn ignored() -> Self {
        Self::Ignored(None)
    }

    pub fn consumed() -> Self {
        Self::Consumed(None)
    }

    #[allow(unused)]
    pub fn callback<C: FnOnce(&mut Composer, &mut Editor) + 'static>(
        mut self,
        callback: C,
    ) -> Self {
        let callback = Box::new(callback);

        match self {
            EventResult::Consumed(ref mut c) | EventResult::Ignored(ref mut c) => {
                *c = Some(callback);
            }
        }

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
        let area = current_surface.area;

        for w in &self.widgets {
            w.draw(area, current_surface, ctx);
        }

        self.surfaces.render(canvas)?;

        Ok(())
    }

    pub fn handle_event(&mut self, event: Event, ctx: &mut Context) -> bool {
        let mut callbacks = Vec::new();

        let mut consumed = false;

        let resized = if let Event::Resize(x, y) = event {
            self.surfaces.resize(Rect::new(0, 0, x, y));
            true
        } else {
            false
        };

        for widget in self.widgets.iter_mut().rev() {
            match widget.handle_event(event, ctx) {
                EventResult::Consumed(Some(cb)) => {
                    consumed = true;
                    callbacks.push(cb);
                }
                EventResult::Consumed(None) => {
                    consumed = true;
                }
                EventResult::Ignored(Some(cb)) => {
                    callbacks.push(cb);
                }
                EventResult::Ignored(None) => {}
            }

            if consumed {
                break;
            }
        }

        for callback in callbacks {
            callback(self, ctx.editor);
        }

        consumed || resized
    }

    pub fn push_widget<W: Widget + 'static>(&mut self, widget: W) {
        self.widgets.push(Box::new(widget));
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

    pub fn render<C: Canvas>(&mut self, canvas: &mut C) -> anyhow::Result<()> {
        let current_surface = &self.surfaces[self.current_surface];
        let prev_surface = &self.surfaces[1 - self.current_surface];

        let diff = prev_surface.diff(current_surface);
        canvas.draw(diff.into_iter())?;
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
