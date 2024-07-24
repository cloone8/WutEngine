use wutengine_core::{
    color::Color,
    component::{Component, ComponentTypeId, DynComponent},
    renderer::RenderContext,
    windowing::WindowIdentifier,
};

use super::ID_CAMERA;

#[derive(Debug)]
pub struct Camera {
    pub display: WindowIdentifier,
    pub clear_color: Color,
}

impl DynComponent for Camera {
    fn get_dyn_component_id(&self) -> ComponentTypeId {
        Self::COMPONENT_ID
    }
}

impl Component for Camera {
    const COMPONENT_ID: ComponentTypeId = ID_CAMERA;
}

impl Camera {
    pub(crate) fn to_context(&self) -> RenderContext {
        RenderContext {
            window: &self.display,
            clear_color: self.clear_color,
        }
    }
}
