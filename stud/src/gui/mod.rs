mod canvas;
mod composer;
mod crossterm_impl;
mod style;
mod surface;

pub use composer::{Composer, EditorWidget};
use crossterm::event::Event;
pub use crossterm_impl::CrosstermCanvas;
pub use style::*;
pub use surface::Cell;

use anyhow::Result;

use crate::editor::Editor;
use crate::jobs::Jobs;

use self::canvas::Canvas;
use self::composer::Context;

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

    pub fn render(&mut self, editor: &mut Editor, jobs: &mut Jobs) -> Result<()> {
        let mut ctx = Context { editor, jobs };
        self.composer.render(&mut self.canvas, &mut ctx)
    }

    pub fn handle_event(&mut self, event: Event, editor: &mut Editor, jobs: &mut Jobs) -> bool {
        let mut ctx = Context { editor, jobs };
        self.composer.handle_event(event, &mut ctx)
    }

    pub fn composer_mut(&mut self) -> &mut Composer {
        &mut self.composer
    }
}
