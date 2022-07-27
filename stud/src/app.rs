use std::{fmt::Write, io};

use crossterm::event::{Event, KeyCode};
use futures_util::{Stream, StreamExt};

use crate::{
    jobs::{Jobs, Outcome},
    output::Output,
};

pub struct App {
    jobs: Jobs,
    output: Output,
    logs: String,
}

impl App {
    pub fn new(output: Output) -> Self {
        Self {
            output,
            jobs: Jobs::new(),
            logs: String::new(),
        }
    }

    pub async fn run(
        &mut self,
        term_events: &mut (impl Stream<Item = Result<Event, io::Error>> + Unpin),
    ) -> crate::Result<()> {
        loop {
            let exit = tokio::select! {
                Some(ev) = term_events.next() => self.on_term_event(ev?)?,
                Some(outcome) = self.jobs.jobs.next() => self.on_job_outcome(outcome?),
            };

            if exit {
                break;
            }
        }

        Ok(())
    }

    fn on_term_event(&mut self, event: Event) -> crate::Result<bool> {
        let _ = write!(self.logs, "event: {event:?}");

        let mut exit = false;
        if let Event::Key(k) = event {
            match k.code {
                KeyCode::Char('l') => {
                    self.output.clear()?;
                    self.output.dump(&self.logs)?;
                }
                KeyCode::Char('q') => exit = true,
                _ => {}
            }
        }

        Ok(exit)
    }

    fn on_job_outcome(&mut self, outcome: Outcome) -> bool {
        let _ = write!(self.logs, "outcome: {outcome:?}");

        outcome.exit
    }
}
