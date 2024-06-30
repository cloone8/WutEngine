use super::KeyType;

pub trait InstanceID {
    fn id(&self) -> KeyType;
}
