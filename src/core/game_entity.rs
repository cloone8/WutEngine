use super::{entity_component::EntityComponent, transform::Transform};

pub struct GameEntity {
    name: String,
    transform: Transform,
    components: Vec<Box<dyn EntityComponent>>,
}

// Public methods for GameEntity
impl GameEntity {
    pub fn builder() -> GameEntityBuilder {
        GameEntityBuilder {
            entity: GameEntity {
                name: String::new(),
                transform: Transform::new(),
                components: Vec::new(),
            }
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

// Engine-internal methods for GameEntity
impl GameEntity {
    pub(crate) fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub(crate) fn get_components(&self) -> &Vec<Box<dyn EntityComponent>> {
        &self.components
    }

    pub(crate) fn get_components_mut(&mut self) -> &mut Vec<Box<dyn EntityComponent>> {
        &mut self.components
    }
}

// Private methods for GameEntity
impl GameEntity {

}

pub struct GameEntityBuilder {
    pub(crate) entity: GameEntity,
}

impl GameEntityBuilder {
    pub fn with_name(mut self, name: &str) -> GameEntityBuilder {
        self.entity.name = name.to_string();
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> GameEntityBuilder {
        self.entity.transform = transform;
        self
    }

    pub fn with_component<T>(mut self, component: T) -> GameEntityBuilder
        where T: EntityComponent + 'static
    {
        self.entity.components.push(Box::new(component));
        self
    }

    pub fn build(self) -> GameEntity {
        self.entity
    }
}
