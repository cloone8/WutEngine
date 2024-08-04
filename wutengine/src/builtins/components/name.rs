use wutengine_core::{Component, ComponentTypeId, DynComponent};

use super::ID_NAME;

#[derive(Debug)]
pub struct Name(String);

impl DynComponent for Name {
    fn get_dyn_component_id(&self) -> ComponentTypeId {
        Self::COMPONENT_ID
    }
}

impl Component for Name {
    const COMPONENT_ID: ComponentTypeId = ID_NAME;
}
