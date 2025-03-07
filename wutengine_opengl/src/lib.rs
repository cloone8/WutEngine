//! The OpenGL rendering backend for WutEngine
//! The basic cross-platform rendering backend

use std::collections::HashMap;
use std::rc::Rc;

use window::Window;
use wutengine_graphics::material::MaterialData;
use wutengine_graphics::mesh::MeshData;
use wutengine_graphics::renderer::{
    HasDisplayHandle, HasWindowHandle, Renderable, RendererMaterialId, RendererMeshId,
    RendererTextureId, Viewport, WindowIdentifier, WutEngineRenderer,
};
use wutengine_graphics::shader::resolver::ShaderResolver;
use wutengine_graphics::texture::TextureData;

mod opengl {
    #![allow(warnings)]
    //! The raw OpenGL generated bindings
    include!(concat!(env!("OUT_DIR"), "/gl_generated_bindings.rs"));
}

mod buffer;
mod error;
mod gltypes;
mod mesh;
mod shader;
mod vao;
mod window;

/// Main OpenGL Renderer
pub struct OpenGLRenderer {
    /// ShaderResolver, responsible for finding shaders on disk
    shader_resolver: Rc<dyn ShaderResolver>,

    /// The active windows
    windows: HashMap<WindowIdentifier, Window>,

    /// All currently active meshes and their data
    meshes: HashMap<RendererMeshId, Option<MeshData>>,

    /// All currently active materials and their data
    materials: HashMap<RendererMaterialId, Option<MaterialData>>,

    /// All currently active textures and their data
    textures: HashMap<RendererTextureId, Option<TextureData>>,
}

impl WutEngineRenderer for OpenGLRenderer {
    fn build<R: ShaderResolver>(shaders: R) -> Self {
        Self {
            shader_resolver: Rc::new(shaders),
            windows: HashMap::default(),
            meshes: HashMap::default(),
            materials: HashMap::default(),
            textures: HashMap::default(),
        }
    }

    fn new_window(
        &mut self,
        id: &WindowIdentifier,
        window: &(impl HasDisplayHandle + HasWindowHandle),
        phys_size: (u32, u32),
    ) {
        log::debug!("Creating window {}", id);

        if self.windows.contains_key(id) {
            log::error!("Cannot create window {} because it already exists", id);
            return;
        }

        let mut new_window = Window::new(self.shader_resolver.clone(), window, phys_size);

        // Insert all currently existing resources to make the window up-to-date
        for (&id, data) in &self.meshes {
            new_window.create_mesh(id);

            if let Some(data) = data.as_ref() {
                new_window.update_mesh(id, data);
            }
        }

        for (&id, data) in &self.materials {
            new_window.create_material(id);

            if let Some(data) = data.as_ref() {
                new_window.update_material(id, data);
            }
        }

        for (&id, data) in &self.textures {
            new_window.create_texture(id);

            if let Some(data) = data.as_ref() {
                new_window.update_texture(id, data);
            }
        }

        self.windows.insert(id.clone(), new_window);
    }

    fn destroy_window(&mut self, id: &WindowIdentifier) {
        log::debug!("Destroying window {}", id);

        let to_destroy = self.windows.remove(id);

        match to_destroy {
            Some(to_destroy) => to_destroy.destroy(),
            None => {
                log::error!("Removing window {} failed because it did not exist", id);
            }
        }
    }

    fn window_size_changed(&mut self, id: &WindowIdentifier, phys_size: (u32, u32)) {
        log::debug!(
            "Handling size change to {}x{} for {}",
            phys_size.0,
            phys_size.1,
            id
        );

        if let Some(window) = self.windows.get_mut(id) {
            window.size_changed(phys_size);
        } else {
            log::error!("Window {} unknown, not doing resize", id);
        }
    }

    fn render(&mut self, render_context: &Viewport, objects: &[Renderable]) {
        log::trace!(
            "Rendering context {:#?} with {} objects",
            render_context,
            objects.len()
        );

        if let Some(window) = self.windows.get_mut(&render_context.window) {
            window.render(render_context, objects);
        } else {
            log::error!(
                "Window {} unknown, not doing rendering",
                render_context.window
            );
        }
    }

    fn create_mesh(&mut self) -> RendererMeshId {
        let id = RendererMeshId::new();

        log::debug!("Creating new mesh with id {}", id);

        self.meshes.insert(id, None);

        for window in self.windows.values_mut() {
            window.create_mesh(id);
        }

        id
    }

    fn delete_mesh(&mut self, id: RendererMeshId) {
        log::debug!("Deleting mesh with id {}", id);

        self.meshes.remove(&id);

        for window in self.windows.values_mut() {
            window.delete_mesh(id);
        }
    }

    fn update_mesh(&mut self, id: RendererMeshId, data: &MeshData) {
        log::debug!("Updating mesh with id {}", id);

        self.meshes.insert(id, Some(data.clone()));

        for window in self.windows.values_mut() {
            window.update_mesh(id, data);
        }
    }

    fn create_texture(&mut self) -> RendererTextureId {
        let id = RendererTextureId::new();

        log::debug!("Creating new texture with id {}", id);

        self.textures.insert(id, None);

        for window in self.windows.values_mut() {
            window.create_texture(id);
        }

        id
    }

    fn delete_texture(&mut self, id: RendererTextureId) {
        log::debug!("Deleting texture with id {}", id);

        for window in self.windows.values_mut() {
            window.delete_texture(id);
        }

        self.textures.remove(&id);
    }

    fn update_texture(&mut self, id: RendererTextureId, data: &TextureData) {
        log::debug!("Updating texture with id {}", id);

        self.textures.insert(id, Some(data.clone()));

        for window in self.windows.values_mut() {
            window.update_texture(id, data);
        }
    }

    fn create_material(&mut self) -> RendererMaterialId {
        let id = RendererMaterialId::new();

        log::debug!("Creating new material with id {}", id);

        self.materials.insert(id, None);

        for window in self.windows.values_mut() {
            window.create_material(id);
        }

        id
    }

    fn delete_material(&mut self, id: RendererMaterialId) {
        log::debug!("Deleting material with id {}", id);

        for window in self.windows.values_mut() {
            window.delete_material(id);
        }

        self.materials.remove(&id);
    }

    fn update_material(&mut self, id: RendererMaterialId, data: &MaterialData) {
        log::debug!("Updating material with id {}", id);

        self.materials.insert(id, Some(data.clone()));

        for window in self.windows.values_mut() {
            window.update_material(id, data);
        }
    }
}

impl Drop for OpenGLRenderer {
    fn drop(&mut self) {
        let window_ids: Vec<_> = self.windows.keys().cloned().collect();

        for window_id in window_ids {
            self.destroy_window(&window_id);
        }
    }
}
