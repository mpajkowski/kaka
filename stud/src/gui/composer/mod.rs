mod dummy;
mod widget;

pub use dummy::DummyWidget;

use stud_core::shapes::Rect;

use self::widget::Widget;

use super::{canvas::Canvas, surface::Surface};

pub struct Composer {
    widgets: Vec<Box<dyn Widget>>,
    surfaces: Surfaces,
}

impl Composer {
    pub fn new(rect: Rect) -> Self {
        let surfaces = Surfaces::new(rect);

        Self {
            surfaces,
            widgets: vec![],
        }
    }

    pub fn surface(&self) -> &Surface {
        self.surfaces.surface()
    }

    pub fn surface_mut(&mut self) -> &mut Surface {
        self.surfaces.surface_mut()
    }

    pub fn render<C: Canvas>(&mut self, canvas: &mut C) -> anyhow::Result<()> {
        let current_surface = self.surfaces.surface_mut();
        let area = current_surface.area;

        for w in &self.widgets {
            w.draw(area, current_surface)?;
        }

        self.surfaces.render(canvas)?;

        Ok(())
    }

    pub fn push_widget<W: Widget + 'static>(&mut self, widget: W) {
        self.widgets.push(Box::new(widget))
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

    pub fn surface(&self) -> &Surface {
        &self.surfaces[self.current_surface]
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
}
