use std::io;

use crate::client::composer::EditorWidget;
use crate::client::Redraw;
use crate::{
    editor::{Buffer, Editor},
    logger, Canvas,
};
use crossterm::event::Event;
use futures_util::{Stream, StreamExt};
use kaka_core::{document::Document, ropey::Rope};
use tokio::sync::mpsc;

use crate::Client;

pub struct App<C> {
    client: Client<C>,
    editor: Editor,
}

impl<C: Canvas> App<C> {
    pub fn new(client: Client<C>) -> Self {
        Self {
            client,
            editor: Editor::init(),
        }
    }

    pub async fn run<
        E: Stream<Item = Result<Event, io::Error>> + Unpin,
        I: Iterator<Item = String>,
    >(
        &mut self,
        args: I,
        term_events: &mut E,
    ) -> anyhow::Result<()> {
        // init logging
        let log_document = Document::new_scratch();
        let buffer = Buffer::new_logging(&log_document);
        let logger_id = buffer.id();
        self.editor.buffers.insert(logger_id, buffer);
        self.editor
            .documents
            .insert(log_document.id(), log_document);
        self.editor.set_logger(logger_id);

        let (log_tx, mut log_rx) = mpsc::unbounded_channel();

        logger::enable(log_tx);

        // open paths from argv
        let mut opened = 0;
        let mut failed = 0;

        for arg in args.skip(1) {
            if let Err(e) = self.editor.open(&*arg, opened == 0) {
                log::error!("{e}");
                failed += 1;
            } else {
                opened += 1;
            }
        }

        log::info!("Opened {opened} documents from args");
        if failed > 0 {
            log::info!("Failed to open {failed} documents");
        }

        // nothing opened (except logs) - create first scratch buffer
        if opened == 0 {
            self.editor.open_scratch(true);
        }

        // push widgets
        self.client
            .composer_mut()
            .push_widget(EditorWidget::default());

        self.render()?;

        // enter event loop
        loop {
            let should_redraw = tokio::select! {
                Some(ev) = term_events.next() => {
                    self.on_term_event(ev?)
                },
                Some(log) = log_rx.recv() => {
                    self.on_log(log)
                }
            };

            let exit = self.editor.should_exit();

            if let Redraw(true) = should_redraw {
                if !exit {
                    self.render()?;
                }
            }

            if exit {
                break;
            }
        }

        Ok(())
    }

    fn on_term_event(&mut self, event: Event) -> Redraw {
        self.client.handle_event(event, &mut self.editor)
    }

    fn on_log(&mut self, log: Rope) -> Redraw {
        self.editor.on_log(log)
    }

    fn render(&mut self) -> anyhow::Result<()> {
        self.client.render(&mut self.editor)
    }
}
