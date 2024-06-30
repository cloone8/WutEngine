use crate::{
    id::{instance::InstanceID, KeyType},
    transform::{LocalTransform, Transform},
};

#[derive(Debug)]
pub struct Object {
    id: KeyType,
    name: String,
    transform: Transform,
}

impl Object {
    pub fn new(id: KeyType, name: impl ToString, transform: LocalTransform) -> Self {
        Self {
            id,
            name: name.to_string(),
            transform: Transform::from_local(transform),
        }
    }
}

impl InstanceID for Object {
    fn id(&self) -> KeyType {
        self.id
    }
}
