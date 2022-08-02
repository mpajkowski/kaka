use std::fmt::Debug;

use stud_core::shapes::Rect;

use super::{canvas::Canvas, surface::Surface};

pub struct Composer {
    surfaces: [Surface; 2],
    current_surface: usize,
}

impl Composer {
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

    pub fn surface_mut(&mut self) -> &Surface {
        &mut self.surfaces[self.current_surface]
    }

    pub fn render<C: Canvas>(&mut self, canvas: &mut C) -> anyhow::Result<()> {
        let current_surface = &self.surfaces[self.current_surface];
        let prev_surface = &self.surfaces[1 - self.current_surface];

        let diff = prev_surface.diff(current_surface);
        canvas.draw(diff.into_iter())?;

        Ok(())
    }
}

impl Debug for Composer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Composer")
            .field("current_surface", &self.current_surface)
            .finish()
    }
}
