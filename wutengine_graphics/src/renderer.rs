use std::rc::Rc;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::color::Color;
use crate::material::MaterialData;
use crate::mesh::MeshData;
use crate::shader::resolver::ShaderResolver;
use crate::windowing::WindowIdentifier;

#[derive(Debug)]
pub struct RenderContext {
    pub window: WindowIdentifier,
    pub clear_color: Color,
}

#[derive(Debug)]
pub struct Renderable {
    pub mesh: Rc<MeshData>,
    pub material: Rc<MaterialData>,
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

    fn render(&mut self, render_context: RenderContext, objects: &[Renderable]);
}
