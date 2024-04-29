use crate::entity::Entity;

#[derive(Debug)]
pub struct World {
    entities: Vec<Entity>
}

impl World {
    pub fn new(entities: Vec<Entity>) -> Self {
        World {
            entities
        }
    }

    pub(crate) fn get_entities(&mut self) -> &mut Vec<Entity> {
        &mut self.entities
    }
}
