//! The various WutEngine renderers and rendering functionality

use wutengine_core::identifiers::WindowIdentifier;
use wutengine_graphics::renderer::{Renderable, Viewport};
use wutengine_graphics::renderer::{
    RendererMaterialId, RendererMeshId, RendererTextureId, WutEngineRenderer,
};
use wutengine_graphics::shader::ShaderResolver;

pub(crate) mod queue;
pub(crate) mod shader_resolver;

#[cfg(feature = "opengl")]
pub use wutengine_opengl::OpenGLRenderer;

use crate::windowing::{HasDisplayHandle, HasWindowHandle};

/// The no-op headless renderer.
/// Ignores all rendering commands, and will
/// leave any opened windows with undefined content.
#[derive(Debug)]
pub struct HeadlessRenderer;

impl WutEngineRenderer for HeadlessRenderer {
    fn new_window(
        &mut self,
        _id: &WindowIdentifier,
        _window: &(impl HasDisplayHandle + HasWindowHandle),
        _phys_size: (u32, u32),
    ) {
    }

    fn window_size_changed(&mut self, _id: &WindowIdentifier, _phys_size: (u32, u32)) {}

    fn destroy_window(&mut self, _id: &WindowIdentifier) {}

    fn render(&mut self, _render_context: &Viewport, _objects: &[Renderable]) {}

    fn build<R: ShaderResolver>(_shaders: R) -> Self {
        Self
    }

    fn create_mesh(&mut self) -> wutengine_graphics::renderer::RendererMeshId {
        RendererMeshId::new()
    }

    fn delete_mesh(&mut self, id: wutengine_graphics::renderer::RendererMeshId) {}

    fn update_mesh(
        &mut self,
        id: wutengine_graphics::renderer::RendererMeshId,
        data: &wutengine_graphics::mesh::MeshData,
    ) {
    }

    fn create_texture(&mut self) -> wutengine_graphics::renderer::RendererTextureId {
        RendererTextureId::new()
    }

    fn delete_texture(&mut self, id: wutengine_graphics::renderer::RendererTextureId) {}

    fn update_texture(
        &mut self,
        id: wutengine_graphics::renderer::RendererTextureId,
        data: &wutengine_graphics::texture::TextureData,
    ) {
    }

    fn create_material(&mut self) -> wutengine_graphics::renderer::RendererMaterialId {
        RendererMaterialId::new()
    }

    fn delete_material(&mut self, id: wutengine_graphics::renderer::RendererMaterialId) {}

    fn update_material(
        &mut self,
        id: wutengine_graphics::renderer::RendererMaterialId,
        data: &wutengine_graphics::material::MaterialData,
    ) {
    }
}
