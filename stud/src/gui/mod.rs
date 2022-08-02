mod canvas;
mod composer;
mod crossterm_impl;
mod style;
mod surface;

pub use composer::{Composer, DummyWidget};
pub use crossterm_impl::CrosstermCanvas;
pub use style::*;
pub use surface::Cell;

use anyhow::Result;

use self::canvas::Canvas;

pub struct Gui<C> {
    canvas: C,
    composer: Composer,
}

impl<C: Canvas> Gui<C> {
    pub fn new(canvas: C) -> Self {
        let dims = canvas.shape();
        let composer = Composer::new(dims);

        Self { canvas, composer }
    }

    pub fn render(&mut self) -> Result<()> {
        self.composer.render(&mut self.canvas)
    }

    pub fn composer_mut(&mut self) -> &mut Composer {
        &mut self.composer
    }
}
