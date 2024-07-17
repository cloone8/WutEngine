use crate::EngineCommand;

mod entity;
mod window;

pub use entity::*;
pub use window::*;

#[derive(Debug)]
pub struct Command {
    pub(crate) commands: Vec<EngineCommand>,
}

/// Internal API
impl Command {
    pub(crate) fn empty() -> Self {
        Command {
            commands: Vec::new(),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.commands.clear();
    }

    pub(crate) fn merge_with(&mut self, other: Self) {
        self.commands.extend(other.commands)
    }

    pub(crate) fn merge(a: Self, b: Self) -> Self {
        let mut new = Command {
            commands: Vec::with_capacity(a.commands.len() + b.commands.len()),
        };

        new.commands.extend(a.commands);
        new.commands.extend(b.commands);

        new
    }

    pub(crate) fn consume(self) -> Vec<EngineCommand> {
        self.commands
    }

    pub(crate) fn len(&self) -> usize {
        self.commands.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

///Public API
impl Command {
    pub fn window(&mut self) -> WindowCommand {
        WindowCommand::new(self)
    }

    pub fn entity(&mut self) -> EntityCommand {
        EntityCommand::new(self)
    }
}
