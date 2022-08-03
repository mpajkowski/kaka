use std::{fmt::Write, io};

use crossterm::event::Event;
use futures_util::{Stream, StreamExt};

use crate::{
    editor::Editor,
    gui::EditorWidget,
    jobs::{Jobs, Outcome},
};

use crate::Gui;

pub struct App {
    jobs: Jobs,
    gui: Gui,
    logs: String,
    editor: Editor,
}

impl App {
    pub fn new(gui: Gui) -> Self {
        Self {
            gui,
            jobs: Jobs::default(),
            logs: String::new(),
            editor: Editor::init(),
        }
    }

    pub async fn run(
        &mut self,
        term_events: &mut (impl Stream<Item = Result<Event, io::Error>> + Unpin),
    ) -> anyhow::Result<()> {
        self.gui.composer_mut().push_widget(EditorWidget::default());

        loop {
            let should_redraw = tokio::select! {
                Some(ev) = term_events.next() => {
                    self.on_term_event(ev?)
                },
                Some(outcome) = self.jobs.jobs.next() => {
                    self.on_job_outcome(outcome?)
                },
            };

            let exit = self.editor.should_exit();

            if should_redraw && !exit {
                self.render()?;
            }

            if exit {
                break;
            }
        }

        Ok(())
    }

    fn on_term_event(&mut self, event: Event) -> bool {
        let _ = writeln!(self.logs, "event: {event:?}");

        self.gui
            .handle_event(event, &mut self.editor, &mut self.jobs)
    }



    fn on_job_outcome(&mut self, outcome: Outcome) -> bool {
        let _ = write!(self.logs, "outcome: {outcome:?}");

        outcome.exit
    }

    fn render(&mut self) -> anyhow::Result<()> {
        self.gui.render(&mut self.editor, &mut self.jobs)
    }
}
