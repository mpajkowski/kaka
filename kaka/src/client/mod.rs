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

use self::composer::Context;

pub use self::canvas::Canvas;

pub struct Client<C> {
    canvas: C,
    composer: Composer,
}

impl<C: Canvas> Client<C> {
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
        if matches!(event, Event::Resize(_, _)) {
            let _ = self.canvas.clear();
        }

        let mut ctx = Context { editor, jobs };
        self.composer.handle_event(event, &mut ctx)
    }

    pub fn composer_mut(&mut self) -> &mut Composer {
        &mut self.composer
    }
}
