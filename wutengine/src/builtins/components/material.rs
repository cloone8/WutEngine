use std::rc::Rc;

use wutengine_core::Component;
use wutengine_graphics::material::MaterialData;

/// A material component, describing how to render
/// a mesh. Works together with the [super::Mesh] component to make
/// an entity renderable.
#[derive(Debug)]
pub struct Material {
    /// The actual material data, in an RC so that
    /// multiple entities can use the same data transparently.
    pub(crate) data: Rc<MaterialData>,
}

impl Component for Material {}

impl Material {
    /// Creates a new material component.
    pub fn new(data: MaterialData) -> Self {
        Self {
            data: Rc::new(data),
        }
    }
}
