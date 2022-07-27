use std::io;

use crossterm::event::Event;
use futures_util::{Stream, StreamExt};

use crate::{
    jobs::{Jobs, Outcome},
    output::Output,
};

pub struct App {
    jobs: Jobs,
    output: Output,
}

impl App {
    pub fn new(output: Output) -> Self {
        Self {
            output,
            jobs: Jobs::new(),
        }
    }

    pub async fn run(
        &mut self,
        term_events: &mut (impl Stream<Item = Result<Event, io::Error>> + Unpin),
    ) -> crate::Result<()> {
        loop {
            tokio::select! {
                Some(ev) = term_events.next() => self.on_term_event(ev?),
                Some(outcome) = self.jobs.jobs.next() => if self.on_job_outcome(outcome?) { break; },
            };
        }

        Ok(())
    }

    fn on_term_event(&mut self, event: Event) {
        eprintln!("event: {event:?}");
    }

    fn on_job_outcome(&mut self, outcome: Outcome) -> bool {
        eprintln!("outcome: {outcome:?}");

        outcome.exit
    }
}
