use std::rc::Rc;
use std::sync::Arc;

use glam::Mat4;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::color::Color;
use crate::material::MaterialData;
use crate::mesh::MeshData;
use crate::shader::resolver::ShaderResolver;
use crate::windowing::WindowIdentifier;

#[derive(Debug)]
pub struct Viewport {
    pub window: WindowIdentifier,
    pub clear_color: Color,
    pub view_mat: Mat4,
    pub projection_mat: Mat4,
}

#[derive(Debug)]
pub struct Renderable {
    pub mesh: Arc<MeshData>,
    pub material: Arc<MaterialData>,
    pub object_to_world: Mat4,
}

pub trait WutEngineRenderer {
    fn build<R: ShaderResolver>(shaders: R) -> Self;

    fn new_window(
        &mut self,
        id: &WindowIdentifier,
        window: &(impl HasDisplayHandle + HasWindowHandle),
        phys_size: (u32, u32),
    );

    fn destroy_window(&mut self, id: &WindowIdentifier);

    fn size_changed(&mut self, id: &WindowIdentifier, phys_size: (u32, u32));

    fn render(&mut self, viewport_context: &Viewport, objects: &[Renderable]);
}
