mod buffer_mgmt;
mod history;
mod insert_mode;
mod mode_switch;
mod movement;
mod text_manipulation;

pub use buffer_mgmt::*;
pub use history::*;
pub use insert_mode::*;
pub use mode_switch::*;
pub use movement::*;
pub use text_manipulation::*;

use std::{borrow::Cow, fmt::Debug};

use crossterm::event::KeyEvent;

use crate::client::composer::{Callback, Widget};

use super::Editor;

pub type CommandFn = fn(&mut CommandData);

pub struct CommandData<'a> {
    pub editor: &'a mut Editor,
    pub trigger: KeyEvent,
    pub count: Option<usize>,
    pub callback: Option<Callback>,
}

impl<'a> CommandData<'a> {
    fn push_widget<W: Widget + 'static>(&mut self, widget: W) {
        self.callback = Some(Box::new(move |composer| {
            composer.push_widget(widget);
        }));
    }
}

#[derive(Clone)]
pub struct Command {
    name: Cow<'static, str>,
    fun: CommandFn,
}

impl Command {
    pub fn new(name: impl Into<Cow<'static, str>>, fun: CommandFn) -> Self {
        Self {
            name: name.into(),
            fun,
        }
    }

    pub fn call(&self, context: &mut CommandData) {
        (self.fun)(context);
    }

    pub fn describe(&self) -> &str {
        &self.name
    }
}

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command").field("name", &self.name).finish()
    }
}
#[macro_export]
macro_rules! command {
    ($fun: ident) => {{
        let name = stringify!($fun);
        Command::new(name, $fun)
    }};
}
