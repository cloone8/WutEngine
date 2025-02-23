use std::sync::Arc;

use glam::Mat4;

pub use raw_window_handle::HasDisplayHandle;
pub use raw_window_handle::HasWindowHandle;
pub use wutengine_core::identifiers::WindowIdentifier;

use crate::color::Color;
use crate::material::MaterialData;
use crate::mesh::MeshData;
use crate::shader::resolver::ShaderResolver;

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
    pub mesh: Arc<MeshData>,

    /// The material
    pub material: Arc<MaterialData>,

    /// The object-to-world matrix
    pub object_to_world: Mat4,
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
    fn size_changed(&mut self, id: &WindowIdentifier, phys_size: (u32, u32));

    /// Render the given objects into the given viewport
    fn render(&mut self, viewport_context: &Viewport, objects: &[Renderable]);
}
