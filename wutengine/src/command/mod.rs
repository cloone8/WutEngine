use crate::EngineCommand;

mod entity;
mod window;

pub use entity::*;
pub use window::*;

#[derive(Debug, Default)]
pub struct Command {
    pub(crate) commands: Vec<EngineCommand>,
}

/// Internal API
impl Command {
    pub const NONE: Self = Self::empty();

    pub(crate) const fn empty() -> Self {
        Command {
            commands: Vec::new(),
        }
    }

    pub(crate) fn merge_with(&mut self, other: Self) {
        self.commands.extend(other.commands)
    }

    pub(crate) fn consume(self) -> Vec<EngineCommand> {
        self.commands
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
