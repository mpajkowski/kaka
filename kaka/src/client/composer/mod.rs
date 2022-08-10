mod editor;
mod widget;

use crossterm::event::Event;
pub use editor::EditorWidget;

use kaka_core::shapes::{Point, Rect};

use crate::{current, editor::Editor};

use self::widget::Widget;

use super::{canvas::Canvas, surface::Surface};

pub struct Context<'a> {
    pub editor: &'a mut Editor,
}

pub struct Composer {
    widgets: Vec<Box<dyn Widget>>,
    surfaces: Surfaces,
    cursor: Point,
}

pub enum EventResult {
    Ignored,
    Consumed,
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

        for w in &self.widgets {
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
        for widget in self.widgets.iter_mut().rev() {
            if let EventResult::Consumed = widget.handle_event(event, ctx) {
                consumed = true;
                break;
            }
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
