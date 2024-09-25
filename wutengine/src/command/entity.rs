use wutengine_core::{Component, EntityId};
use wutengine_ecs::world::World;

use crate::EngineCommand;

use super::Command;

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
    pub fn spawn(&mut self, callback: for<'x> fn(EntityId, &'x mut World)) {
        self.cmd.commands.push(EngineCommand::SpawnEntity(callback));
    }
}
