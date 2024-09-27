use super::Command;

mod builder;

pub use builder::*;
use wutengine_core::EntityId;

#[repr(transparent)]
pub struct EntityCommand<'a> {
    cmd: &'a mut Command,
}

impl<'a> EntityCommand<'a> {
    pub(crate) fn new(cmd: &'a mut Command) -> Self {
        Self { cmd }
    }
}

impl<'a> EntityCommand<'a> {
    pub fn spawn(&'a mut self) -> EntityBuilder<'a> {
        EntityBuilder::new(self)
    }

    pub fn destroy(&'a mut self, id: EntityId) {
        self.cmd
            .commands
            .push(crate::EngineCommand::DestroyEntity(id));
    }
}
