use std::rc::Rc;

use wutengine_core::Component;
use wutengine_graphics::material::MaterialData;

#[derive(Debug)]
pub struct Material {
    pub(crate) data: Rc<MaterialData>,
}

impl Component for Material {}

impl Material {
    pub fn new(data: MaterialData) -> Self {
        Self {
            data: Rc::new(data),
        }
    }
}
