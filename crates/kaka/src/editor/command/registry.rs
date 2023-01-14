use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, Weak},
};

use super::*;

macro_rules! command {
    ($fun: ident, $typable: expr, $mappable: expr, [$($alias: expr),* $(,)?]) => {{
        let name = stringify!($fun);
        Command::new(name, $fun, $typable, $mappable, vec![$($alias),*] as Vec<&'static str>)
    }};

    ($fun: ident, $typable: expr, $mappable: expr $(,)?) => { command!($fun, $typable, $mappable, []) };

    ($fun: ident, [$($alias: expr),* ]) => { command!($fun, true, true, [$($alias),*]) };

    ($fun: ident) => { command!($fun, true, true) };
}

#[derive(Debug, Default)]
pub struct Registry {
    commands: HashMap<Cow<'static, str>, Weak<Command>>,
    typable_idx: HashMap<Cow<'static, str>, Arc<Command>>,
    mappable_idx: HashMap<Cow<'static, str>, Arc<Command>>,
}

impl Registry {
    pub fn register(&mut self, command: Command) {
        assert!(command.typable || command.mappable, "Useless command");

        let command = Arc::new(command);

        if command.typable {
            self.typable_idx
                .insert(command.name.clone(), Arc::clone(&command));
        }

        if command.mappable {
            self.mappable_idx
                .insert(command.name.clone(), Arc::clone(&command));
        }

        self.commands
            .insert(command.name().clone(), Arc::downgrade(&command));
    }

    pub fn command_by_name(
        &self,
        name: &str,
        typable_required: bool,
        mappable_required: bool,
    ) -> Option<Arc<Command>> {
        let cmd = match (typable_required, mappable_required) {
            (true, false) => return self.typable_idx.get(name).cloned(),
            (false, true) => return self.mappable_idx.get(name).cloned(),
            _ => self.commands.get(name)?.upgrade()?,
        };

        if mappable_required && !cmd.mappable() {
            return None;
        }

        if typable_required && !cmd.typable() {
            return None;
        }

        Some(Arc::clone(&cmd))
    }

    pub fn populate() -> Self {
        let mut this = Self::default();

        let commands = [
            command!(switch_to_normal_mode),
            command!(switch_to_insert_mode_inplace),
            command!(switch_to_insert_mode_line_start),
            command!(switch_to_insert_mode_after),
            command!(switch_to_insert_mode_line_end),
            command!(move_left),
            command!(move_down),
            command!(move_up),
            command!(move_right),
            command!(goto_line_default_top),
            command!(delete_line),
            command!(goto_line_default_bottom),
            command!(undo),
            command!(redo),
            command!(save, ["w"]),
            command!(close, ["q"]),
            command!(remove_char),
            command!(command_mode, false, true),
            command!(buffer_next),
            command!(buffer_prev),
            command!(buffer_create),
            command!(buffer_kill),
        ];

        for cmd in commands {
            this.register(cmd);
        }

        this
    }
}
