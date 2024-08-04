use std::rc::Rc;

use wutengine_core::{Component, ComponentTypeId, DynComponent};
use wutengine_graphics::material::MaterialData;

use super::ID_MATERIAL;

#[derive(Debug)]
pub struct Material {
    pub(crate) data: Rc<MaterialData>,
}

impl DynComponent for Material {
    fn get_dyn_component_id(&self) -> ComponentTypeId {
        Self::COMPONENT_ID
    }
}

impl Component for Material {
    const COMPONENT_ID: ComponentTypeId = ID_MATERIAL;
}

impl Material {
    pub fn new(data: MaterialData) -> Self {
        Self {
            data: Rc::new(data),
        }
    }
}
