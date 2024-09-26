use wutengine_core::Component;
use wutengine_ecs::Dynamic;

use super::EntityCommand;

#[must_use = "The entity is not build until `.build()` is called"]
pub struct EntityBuilder<'a> {
    entity_cmd: &'a mut EntityCommand<'a>,
    components: Vec<Dynamic>,
}

impl<'a> EntityBuilder<'a> {
    pub(crate) fn new(entity_commands: &'a mut EntityCommand<'a>) -> Self {
        Self {
            entity_cmd: entity_commands,
            components: Vec::new(),
        }
    }

    pub fn with_component<T: Component>(&mut self, component: T) -> &mut Self {
        self.components.push(Dynamic::new(component));

        self
    }

    pub fn build(self) {
        self.entity_cmd
            .cmd
            .commands
            .push(crate::EngineCommand::SpawnEntity(self.components));
    }
}
