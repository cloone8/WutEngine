use wutengine_core::{DynComponent, EntityId};

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
    pub fn spawn(&mut self) -> EntityId {
        let new_id = EntityId::random();

        self.cmd
            .commands
            .push(EngineCommand::SpawnEntity(new_id, Vec::new()));

        new_id
    }

    pub fn spawn_with_components(&mut self, components: Vec<Box<dyn DynComponent>>) -> EntityId {
        let new_id = EntityId::random();

        self.cmd
            .commands
            .push(EngineCommand::SpawnEntity(new_id, components));

        new_id
    }
}
