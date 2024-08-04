use std::collections::HashMap;

use archetype::Archetype;
use vec::AnyVec;
use wutengine_core::EntityId;

pub mod archetype;
pub mod vec;

#[derive(Debug)]
pub struct ArchetypeStorage {
    entity_ids: Vec<EntityId>,
    storages: Box<[AnyVec]>,
}

#[derive(Debug)]
pub struct ComponentStorage {
    archetypes: HashMap<Archetype, ArchetypeStorage>,
}
