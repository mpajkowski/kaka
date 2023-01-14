mod buffer_mgmt;
mod history;
mod insert_mode;
mod mode_switch;
mod movement;
pub mod registry;
mod text_manipulation;

pub use buffer_mgmt::*;
pub use history::*;
pub use insert_mode::*;
pub use mode_switch::*;
pub use movement::*;
pub use text_manipulation::*;

pub use registry::Registry as CommandRegistry;

use std::{borrow::Cow, fmt::Debug};

use crate::client::composer::{Callback, Widget};

use super::Editor;

pub type CommandFn = fn(&mut CommandData);

pub struct CommandData<'a> {
    pub editor: &'a mut Editor,
    pub count: Option<usize>,
    pub callback: Option<Callback>,
}

impl<'a> CommandData<'a> {
    pub fn push_widget<W: Widget + 'static>(&mut self, widget: W) {
        self.callback = Some(Box::new(move |composer| {
            composer.push_widget(widget);
        }));
    }
}

#[derive(Clone)]
pub struct Command {
    name: Cow<'static, str>,
    aliases: Vec<Cow<'static, str>>,
    typable: bool,
    mappable: bool,
    fun: CommandFn,
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.aliases == other.aliases
            && self.typable == other.typable
            && self.mappable == other.mappable
            && std::ptr::eq(
                self.fun as *const fn(&mut CommandData),
                other.fun as *const _,
            )
    }
}

impl Command {
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        fun: CommandFn,
        typable: bool,
        mappable: bool,
        aliases: impl IntoIterator<Item = impl Into<Cow<'static, str>>>,
    ) -> Self {
        Self {
            name: name.into(),
            aliases: aliases.into_iter().map(|a| a.into()).collect(),
            fun,
            mappable,
            typable,
        }
    }

    pub fn call(&self, context: &mut CommandData) {
        (self.fun)(context);
    }

    pub const fn name(&self) -> &Cow<'static, str> {
        &self.name
    }

    pub const fn typable(&self) -> bool {
        self.typable
    }

    pub const fn mappable(&self) -> bool {
        self.mappable
    }

    pub fn aliases(&self) -> &[Cow<'static, str>] {
        &self.aliases
    }
}

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("name", &self.name)
            .field("aliases", &self.aliases)
            .field("fun", &(self.fun as *const CommandFn))
            .finish()
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    use kaka_core::{document::Document, ropey::Rope};

    use crate::{
        current,
        editor::{Buffer, Editor},
    };

    // to save characters typed :P
    pub type B<'a> = &'a Buffer;
    pub type D<'a> = &'a Document;

    pub fn test_cmd<C: FnOnce(&Buffer, &Document)>(
        start_position: usize,
        text: impl AsRef<str>,
        command: fn(&mut CommandData),
        check: C,
    ) {
        let mut editor = Editor::init();

        let mut document = Document::new_scratch();
        *document.text_mut() = Rope::from(text.as_ref());

        let buffer = Buffer::new_text(start_position, &document).unwrap();

        editor.add_buffer_and_document(buffer, document, true);

        let mut data = CommandData {
            editor: &mut editor,
            count: Some(1),
            callback: None,
        };

        command(&mut data);

        let (buf, doc) = current!(data.editor);

        check(buf, doc);
    }
}
