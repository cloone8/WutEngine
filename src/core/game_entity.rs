use super::{entity_component::EntityComponent, transform::Transform};

pub struct GameEntity {
    name: String,
    transform: Transform,
    components: Vec<Box<dyn EntityComponent>>,
    children: Vec<GameEntity>,
}

// Public methods for GameEntity
impl GameEntity {
    pub fn builder() -> GameEntityBuilder<No, No> {
        GameEntityBuilder {
            entity: GameEntity {
                name: String::new(),
                transform: Transform::new(),
                components: Vec::new(),
                children: Vec::new(),
            },
            named_marker: std::marker::PhantomData,
            placed_marker: std::marker::PhantomData,
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

    pub(crate) fn get_children(&self) -> &Vec<GameEntity> {
        &self.children
    }

    pub(crate) fn get_children_mut(&mut self) -> &mut Vec<GameEntity> {
        &mut self.children
    }
}

// Private methods for GameEntity
impl GameEntity {

}

pub trait AssignedState {}
pub struct Yes;
pub struct No;

impl AssignedState for Yes { }
impl AssignedState for No { }

pub struct GameEntityBuilder<Named: AssignedState, Placed: AssignedState> {
    pub(crate) entity: GameEntity,
    named_marker: std::marker::PhantomData<Named>,
    placed_marker: std::marker::PhantomData<Placed>,
}

impl<P: AssignedState> GameEntityBuilder<No, P> {
    pub fn with_name(self, name: &str) -> GameEntityBuilder<Yes, P> {
        GameEntityBuilder {
            entity: GameEntity {
                name: name.to_string(),
                ..self.entity
            },
            named_marker: std::marker::PhantomData,
            placed_marker: std::marker::PhantomData,
        }
    }
}

impl<P: AssignedState> GameEntityBuilder<P, No> {
    pub fn with_transform(self, transform: Transform) -> GameEntityBuilder<P, Yes> {
        GameEntityBuilder {
            entity: GameEntity {
                transform,
                ..self.entity
            },
            named_marker: std::marker::PhantomData,
            placed_marker: std::marker::PhantomData,
        }
    }
}

impl GameEntityBuilder<Yes, Yes> {
    pub fn build(self) -> GameEntity {
        self.entity
    }
}

impl<N: AssignedState, P: AssignedState> GameEntityBuilder<N, P> {
    pub fn with_component<T>(mut self, component: T) -> GameEntityBuilder<N, P>
        where T: EntityComponent + 'static
    {
        self.entity.components.push(Box::new(component));
        self
    }

    pub fn with_child(mut self, child: GameEntity) -> GameEntityBuilder<N, P> {
        self.entity.children.push(child);
        self
    }
}
