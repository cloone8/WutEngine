use glam::Mat4;

pub use raw_window_handle::HasDisplayHandle;
pub use raw_window_handle::HasWindowHandle;
pub use wutengine_core::identifiers::WindowIdentifier;
use wutengine_util_macro::generate_atomic_id;

use crate::color::Color;
use crate::material::MaterialData;
use crate::mesh::MeshData;
use crate::shader::resolver::ShaderResolver;
use crate::texture::TextureData;

/// A general descriptor for a viewport
#[derive(Debug)]
pub struct Viewport {
    /// The viewport window
    pub window: WindowIdentifier,

    /// The viewport clear color
    pub clear_color: Color,

    /// The viewport view matrix
    pub view_mat: Mat4,

    /// The viewport projection matrix
    pub projection_mat: Mat4,
}

/// A renderable object
#[derive(Debug)]
pub struct Renderable {
    /// The mesh
    pub mesh: RendererMeshId,

    /// The material
    pub material: RendererMaterialId,

    /// The object-to-world matrix
    pub object_to_world: Mat4,
}

generate_atomic_id! {
    /// An identifier for a mesh in a [WutEngineRenderer]
    RendererMeshId
}

generate_atomic_id! {
    /// An identifier for a texture in a [WutEngineRenderer]
    RendererTextureId
}

generate_atomic_id! {
    /// An identifier for a material in a [WutEngineRenderer]
    RendererMaterialId
}

/// A rendering backend for WutEngine
pub trait WutEngineRenderer {
    /// Build a new rendering backend with the given resolver
    fn build<R: ShaderResolver>(shaders: R) -> Self;

    /// Initialize a new window for rendering, with the given id, native handle, and
    /// physical size in pixels
    fn new_window(
        &mut self,
        id: &WindowIdentifier,
        window: &(impl HasDisplayHandle + HasWindowHandle),
        phys_size: (u32, u32),
    );

    /// Destroy all resources corresponding to the given identifier
    fn destroy_window(&mut self, id: &WindowIdentifier);

    /// Called by the engine when the size of the window has changed
    fn window_size_changed(&mut self, id: &WindowIdentifier, phys_size: (u32, u32));

    /// Creates a new (uninitialized) mesh
    fn create_mesh(&mut self) -> RendererMeshId;

    /// Deletes an existing mesh
    fn delete_mesh(&mut self, id: RendererMeshId);

    /// Updates the data for the given mesh
    fn update_mesh(&mut self, id: RendererMeshId, data: &MeshData);

    /// Creates a new (uninitialized) texture
    fn create_texture(&mut self) -> RendererTextureId;

    /// Deletes an existing texture
    fn delete_texture(&mut self, id: RendererTextureId);

    /// Updates the data for the given texture
    fn update_texture(&mut self, id: RendererTextureId, data: &TextureData);

    /// Creates a new (uninitialized) material
    fn create_material(&mut self) -> RendererMaterialId;

    /// Deletes an existing material
    fn delete_material(&mut self, id: RendererMaterialId);

    /// Updates the data for the givenmaterial
    fn update_material(&mut self, id: RendererMaterialId, data: &MaterialData);

    /// Render the given objects into the given viewport
    fn render(&mut self, viewport_context: &Viewport, objects: &[Renderable]);
}
