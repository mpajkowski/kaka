use std::{borrow::Cow, collections::HashMap, sync::Arc};

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
    typable: HashMap<Cow<'static, str>, Arc<Command>>,
    mappable: HashMap<Cow<'static, str>, Arc<Command>>,
}

impl Registry {
    pub fn register(&mut self, command: Command) {
        assert!(command.typable || command.mappable, "Useless command");

        let command = Arc::new(command);

        if command.typable() {
            self.typable
                .insert(command.name().clone(), Arc::clone(&command));

            for alias in command.aliases() {
                self.typable.insert(alias.clone(), Arc::clone(&command));
            }
        }

        if command.mappable() {
            self.mappable
                .insert(command.name.clone(), Arc::clone(&command));
        }
    }

    pub fn mappable_command_by_name(&self, name: &str) -> Option<Arc<Command>> {
        self.mappable.get(name).cloned()
    }

    pub fn typable_command_by_name(&self, name: &str) -> Option<Arc<Command>> {
        self.typable.get(name).cloned()
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

#[cfg(test)]
mod test {
    use super::*;

    fn dummy(_: &mut CommandData) {}

    #[test]
    fn command_macro() {
        let command = command!(dummy);

        assert_eq!(command.name(), "dummy");
        assert!(command.typable());
        assert!(command.mappable());

        let command = command!(dummy, true, false);
        assert!(command.typable());
        assert!(!command.mappable());

        let command = command!(dummy, false, true);
        assert!(!command.typable());
        assert!(command.mappable());

        let command = command!(dummy, ["x", "d"]);
        assert_eq!(command.aliases(), &["x", "d"]);
    }

    #[test]
    fn get_typable_command() {
        let mut registry = Registry::default();
        let command = command!(dummy);
        registry.register(command.clone());

        let command_ptr = registry.typable_command_by_name("dummy").unwrap();
        assert_eq!(command, *command_ptr);
    }

    #[test]
    fn get_mappable_command() {
        let mut registry = Registry::default();
        let command = command!(dummy);
        registry.register(command.clone());

        let command_ptr = registry.mappable_command_by_name("dummy").unwrap();
        assert_eq!(command, *command_ptr);
    }

    #[test]
    fn get_command_by_alias() {
        let mut registry = Registry::default();
        let command = command!(dummy, ["x", "d"]);
        registry.register(command.clone());

        let command_ptr = registry.typable_command_by_name("x").unwrap();
        assert_eq!(command, *command_ptr);

        let command_ptr = registry.typable_command_by_name("d").unwrap();
        assert_eq!(command, *command_ptr);
    }
}
