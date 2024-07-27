use std::rc::Rc;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::color::Color;
use crate::material::MaterialData;
use crate::mesh::MeshData;
use crate::windowing::WindowIdentifier;

#[derive(Debug)]
pub struct RenderContext<'a> {
    pub window: &'a WindowIdentifier,
    pub clear_color: Color,
}

pub struct Renderable {
    pub mesh: Rc<MeshData>,
    pub material: Rc<MaterialData>,
}

pub trait WutEngineRenderer: Default {
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
