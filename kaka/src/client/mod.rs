pub mod canvas;
pub mod composer;
pub mod style;
pub mod surface;

mod crossterm_impl;

use crossterm::event::Event;
pub use crossterm_impl::CrosstermCanvas;

use anyhow::Result;

use crate::editor::Editor;

use self::composer::{Composer, Context};

pub use self::canvas::Canvas;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Redraw(pub bool);

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

    pub fn render(&mut self, editor: &mut Editor) -> Result<()> {
        let mut ctx = Context { editor };
        self.composer.render(&mut self.canvas, &mut ctx)
    }

    pub fn handle_event(&mut self, event: Event, editor: &mut Editor) -> Redraw {
        if matches!(event, Event::Resize(_, _)) {
            let _ = self.canvas.clear();
        }

        let mut ctx = Context { editor };
        self.composer.handle_event(event, &mut ctx)
    }

    pub fn composer_mut(&mut self) -> &mut Composer {
        &mut self.composer
    }
}
