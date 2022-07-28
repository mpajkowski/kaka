use std::{fmt::Write, io};

use crossterm::event::{Event, KeyCode, KeyEvent};
use futures_util::{Stream, StreamExt};

use crate::{
    editor::{Editor, KeymapTreeElement},
    jobs::{Jobs, Outcome},
    output::Output,
};

pub struct App {
    jobs: Jobs,
    output: Output,
    logs: String,
    editor: Editor,
}

impl App {
    pub fn new(output: Output) -> Self {
        Self {
            output,
            jobs: Jobs::new(),
            logs: String::new(),
            editor: Editor::new(),
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
            self.on_key_event(k)
        }

        Ok(exit)
    }

    fn on_key_event(&mut self, event: KeyEvent) {
        let call = {
            let (chain, keymap_element) = {
                let buffer = self.editor.buffers.get_mut(&self.editor.current).unwrap();
                let keymap = buffer.keymap();

                if let Some(buf1) = self.editor.buffered_keys.first() {
                    (true, keymap.feed(*buf1))
                } else {
                    (false, keymap.feed(event))
                }
            };

            let mut keymap_element = match keymap_element {
                Some(ke) => ke,
                None => return,
            };

            for buf_key in self.editor.buffered_keys.iter().skip(1) {
                keymap_element = match keymap_element {
                    KeymapTreeElement::Node(k) => k.feed(*buf_key).unwrap(),
                    _ => unreachable!(),
                };
            }

            let mut call = None;
            match keymap_element {
                KeymapTreeElement::Node(n) if chain => match n.feed(event) {
                    Some(KeymapTreeElement::Leaf(command)) => {
                        call = Some(command.clone());
                        self.editor.buffered_keys.clear();
                    }
                    Some(KeymapTreeElement::Node(_)) => self.editor.buffered_keys.push(event),
                    None => self.editor.buffered_keys.clear(),
                },
                KeymapTreeElement::Node(_) => self.editor.buffered_keys.push(event),
                KeymapTreeElement::Leaf(command) => {
                    call = Some(command.clone());
                    self.editor.buffered_keys.clear();
                }
            };
            call
        };

        if let Some(call) = call {
            call.call(&mut self.editor, &mut self.output);
        }
    }

    fn on_job_outcome(&mut self, outcome: Outcome) -> bool {
        let _ = write!(self.logs, "outcome: {outcome:?}");

        outcome.exit
    }
}
