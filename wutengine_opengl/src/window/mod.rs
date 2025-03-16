//! Module for an OpenGL window and associated context. Most of the main code of the backend is here.

use core::ptr::null_mut;
use std::collections::HashMap;
use std::rc::Rc;

use gl_from_raw_window_handle::{GlConfig, GlContext, Profile};
use glam::{Mat4, Vec4};
use thiserror::Error;
use wutengine_graphics::material::{MaterialData, get_default_texture};
use wutengine_graphics::mesh::MeshData;
use wutengine_graphics::renderer::{
    HasDisplayHandle, HasWindowHandle, Renderable, RendererMaterialId, RendererMeshId,
    RendererTextureId, Viewport,
};
use wutengine_graphics::shader::ShaderId;
use wutengine_graphics::shader::ShaderResolver;
use wutengine_graphics::texture::TextureData;

use crate::error::checkerr;
use crate::mesh::{GlMeshBuffers, index_type_to_gl};
use crate::opengl::types::{GLint, GLuint};
use crate::opengl::{self, Gl};
use crate::shader::{self, GlShaderProgram};
use crate::texture::GlTexture;
use crate::texture::tex2d::GlTexture2D;
use crate::vao::Vao;

/// An OpenGL representation of a rendering window, with a unique context
/// and set of GPU resources
pub(crate) struct Window {
    shader_resolver: Rc<dyn ShaderResolver>,
    context: GlContext,
    bindings: Gl,

    default_texture: GlTexture,

    // === Resources ===
    /// Shaders
    shaders: HashMap<ShaderId, GlShaderProgram>,

    /// Meshes
    meshes: HashMap<RendererMeshId, GlMeshBuffers>,

    /// Textures
    textures: HashMap<RendererTextureId, GlTexture>,

    /// Materials
    materials: HashMap<RendererMaterialId, MaterialData>,

    /// VAOs
    attributes: HashMap<RendererMeshId, Vao>,
}

impl Window {
    /// Creates a new window-specific context for the given native handle and initial size.
    /// Uses the provided shader resolver to find the shaders on disk.
    pub(crate) fn new(
        shader_resolver: Rc<dyn ShaderResolver>,
        handles: impl HasDisplayHandle + HasWindowHandle,
        size: (u32, u32),
    ) -> Self {
        let context = unsafe {
            GlContext::create(
                &handles,
                GlConfig {
                    version: (4, 1),
                    profile: Profile::Core,
                    red_bits: 8,
                    blue_bits: 8,
                    green_bits: 8,
                    alpha_bits: 8,
                    depth_bits: 24,
                    stencil_bits: 8,
                    samples: None,
                    srgb: true,
                    double_buffer: true,
                    vsync: false,
                },
            )
            .unwrap()
        };

        unsafe {
            context.make_current();
        }

        let bindings = Gl::load_with(|s| context.get_proc_address(s));

        unsafe {
            bindings.Viewport(0, 0, size.0.try_into().unwrap(), size.1.try_into().unwrap());
            bindings.Enable(opengl::DEPTH_TEST);
        };

        checkerr!(&bindings);

        // We inject the default texture here
        let mut default_texture = GlTexture::Tex2D(GlTexture2D::new(&bindings).unwrap());
        default_texture.upload_data(&bindings, &get_default_texture::<16>());

        Self {
            shader_resolver,
            context,
            bindings,
            default_texture,
            shaders: Default::default(),
            meshes: Default::default(),
            textures: Default::default(),
            materials: Default::default(),
            attributes: Default::default(),
        }
    }

    /// Destroys this [Window], its associated OpenGL context, and all resources
    pub(crate) fn destroy(mut self) {
        unsafe {
            self.context.make_current();
        }

        let mesh_ids: Vec<_> = self.meshes.keys().copied().collect();
        let material_ids: Vec<_> = self.materials.keys().copied().collect();
        let texture_ids: Vec<_> = self.textures.keys().copied().collect();
        let shader_ids: Vec<_> = self.shaders.keys().cloned().collect();

        mesh_ids.into_iter().for_each(|id| self.delete_mesh(id));

        material_ids
            .into_iter()
            .for_each(|id| self.delete_material(id));

        texture_ids
            .into_iter()
            .for_each(|id| self.delete_texture(id));

        shader_ids.into_iter().for_each(|id| {
            let sh = self.shaders.remove(&id).unwrap();
            sh.destroy(&self.bindings);
        });

        self.default_texture.destroy(&self.bindings);
    }

    /// A function to be called whenever the size of the native window changed. Changes
    /// the OpenGL rendering viewport
    pub(crate) fn size_changed(&mut self, size: (u32, u32)) {
        unsafe {
            self.context.make_current();

            self.bindings
                .Viewport(0, 0, size.0 as GLint, size.1 as GLint);

            checkerr!(&self.bindings);
        };
    }

    /// Renders the given objects with the provided rendering context.
    /// The context holds the base view and projection matrices, as well as the
    /// viewport configuration. The objects represent the meshes to render, as well as which
    /// shaders and model matrices to use for rendering them.
    pub(crate) fn render(&mut self, viewport_context: &Viewport, objects: &[Renderable]) {
        unsafe {
            self.context.make_current();
        }

        // First we transform the projection matrix depth range from the universal [0..1] to OpenGL [-1..1]
        let projection_mat = viewport_context.projection_mat
            * Mat4::from_cols(
                Vec4::new(1.0, 0.0, 0.0, 0.0),
                Vec4::new(0.0, 1.0, 0.0, 0.0),
                Vec4::new(0.0, 0.0, 2.0, 0.0),
                Vec4::new(0.0, 0.0, -1.0, 1.0),
            );

        let gl = &self.bindings;

        let clear_color = viewport_context.clear_color;

        unsafe {
            gl.ClearColor(clear_color.r, clear_color.g, clear_color.b, clear_color.a);
            checkerr!(gl);
            gl.Clear(opengl::COLOR_BUFFER_BIT | opengl::DEPTH_BUFFER_BIT);
            checkerr!(gl);
        }

        for object in objects {
            let mesh = match self.meshes.get_mut(&object.mesh) {
                Some(m) => m,
                None => {
                    log::error!("Missing mesh buffers for mesh {}", object.mesh);
                    continue;
                }
            };

            let vao = match self.attributes.get_mut(&object.mesh) {
                Some(m) => m,
                None => {
                    log::error!("Missing mesh VAO for mesh {}", object.mesh);
                    continue;
                }
            };

            let material = match self.materials.get(&object.material) {
                Some(m) => m,
                None => {
                    log::error!("Missing material {}", object.material);
                    continue;
                }
            };

            let shader_id = material.shader.as_ref();
            if shader_id.is_none() {
                log::trace!(
                    "Not rendering object because its material ({}) has no shader attached",
                    object.material
                );
                continue;
            }

            let shader_id = shader_id.unwrap();

            let shader = match self.shaders.get_mut(shader_id) {
                Some(sh) => sh,
                None => {
                    log::error!("Missing shader {}", shader_id);
                    continue;
                }
            };

            // Check if the VAO attributes are still up-to-date. Set them if not
            if !vao.layout_matches(&mesh.vertex_layout, shader.get_vertex_layout()) {
                vao.set_layout(gl, mesh, shader.get_vertex_layout().clone());
            }

            vao.bind(gl);
            shader.use_program(gl);

            // Set the uniforms
            let mut first_free_texture_unit = 0;

            shader.set_uniform_defaults(
                gl,
                &material.parameters,
                &mut first_free_texture_unit,
                &mut self.default_texture,
            );

            shader.set_uniforms(
                gl,
                &material.parameters,
                &mut first_free_texture_unit,
                &mut self.textures,
            );

            if first_free_texture_unit > 0 {
                log::trace!("Bound {} texture units", first_free_texture_unit);
            }

            // Set MVP matrices
            shader.set_mvp(
                gl,
                object.object_to_world,
                viewport_context.view_mat,
                projection_mat,
            );

            unsafe {
                gl.DrawElements(
                    index_type_to_gl(mesh.element_type),
                    GLuint::try_from(mesh.num_elements).unwrap() as GLint,
                    mesh.index_size,
                    null_mut(),
                );

                checkerr!(gl);
            }

            vao.unbind(gl);
        }

        self.context.swap_buffers();
    }
}

/// Resource management
impl Window {
    /// Creates new OpenGL buffers for the given mesh and registers it with the
    /// context
    pub(crate) fn create_mesh(&mut self, id: RendererMeshId) {
        log::trace!("Creating mesh {}", id);

        let buffers = match GlMeshBuffers::new(&self.bindings) {
            Ok(b) => b,
            Err(e) => {
                log::error!("Failed to create OpenGL mesh buffers: {:#?}", e);
                return;
            }
        };

        debug_assert!(!self.meshes.contains_key(&id), "Mesh ID already exists");

        let attributes = match Vao::new(&self.bindings) {
            Ok(a) => a,
            Err(e) => {
                log::error!("Failed to create OpenGL VAO: {:#?}", e);
                buffers.destroy(&self.bindings);
                return;
            }
        };

        debug_assert!(!self.attributes.contains_key(&id), "VAO already exists");

        self.meshes.insert(id, buffers);
        self.attributes.insert(id, attributes);
    }

    /// Deletes the GPU resources for the given mesh
    pub(crate) fn delete_mesh(&mut self, id: RendererMeshId) {
        log::trace!("Deleting mesh {}", id);

        let vao = self.attributes.remove(&id);

        match vao {
            Some(vao) => vao.destroy(&self.bindings),
            None => log::warn!("Tried to delete non-existing mesh VAO for mesh {}", id),
        };

        let buffers = self.meshes.remove(&id);

        match buffers {
            Some(buffers) => buffers.destroy(&self.bindings),
            None => log::warn!("Tried to delete non-existing mesh buffers for mesh {}", id),
        };
    }

    /// Uploads new data to the GPU for the given mesh
    pub(crate) fn update_mesh(&mut self, id: RendererMeshId, data: &MeshData) {
        log::trace!("Updating mesh {}", id);

        let buffers = self.meshes.get_mut(&id);

        if buffers.is_none() {
            log::error!("Trying to update data of unknown mesh {}", id);
            return;
        }

        let buffers = buffers.unwrap();

        buffers.upload_data(&self.bindings, data);
    }

    /// Generates OpenGL buffers for the given texture and registers it with the context
    pub(crate) fn create_texture(&mut self, id: RendererTextureId) {
        log::trace!("Creating texture {}", id);

        debug_assert!(
            !self.textures.contains_key(&id),
            "Texture ID already exists"
        );

        let texture = match GlTexture2D::new(&self.bindings) {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to create OpenGL texture: {:#?}", e);
                return;
            }
        };

        self.textures.insert(id, GlTexture::Tex2D(texture));
    }

    /// Deletes the given texture and its associated GPU resources in this context
    pub(crate) fn delete_texture(&mut self, id: RendererTextureId) {
        log::trace!("Deleting texture {}", id);

        let texture = self.textures.remove(&id);

        match texture {
            Some(t) => t.destroy(&self.bindings),
            None => {
                log::warn!("Tried to delete non-existing texture {}", id)
            }
        };
    }

    /// Uploads new texture data for the given texture ID
    pub(crate) fn update_texture(&mut self, id: RendererTextureId, data: &TextureData) {
        log::trace!("Updating texture {}", id);

        let texture = self.textures.get_mut(&id);

        if texture.is_none() {
            log::error!("Trying to update unknown texture {}", id);
            return;
        }

        let texture = texture.unwrap();

        texture.upload_data(&self.bindings, data);
    }

    /// Registers the given material with this OpenGL window and context
    pub(crate) fn create_material(&mut self, id: RendererMaterialId) {
        log::trace!("Creating material {}", id);

        debug_assert!(
            !self.materials.contains_key(&id),
            "Material ID already exists"
        );

        self.materials.insert(id, MaterialData::default());
    }

    /// Removes the given material from this OpenGL window and context
    pub(crate) fn delete_material(&mut self, id: RendererMaterialId) {
        log::trace!("Deleting material {}", id);

        let materialdata = self.materials.remove(&id);

        if materialdata.is_none() {
            log::warn!("Tried to delete non-existing material {}", id);
        }
    }

    /// Updates the given material
    pub(crate) fn update_material(&mut self, id: RendererMaterialId, data: &MaterialData) {
        log::trace!("Updating material {}", id);

        match self.materials.get_mut(&id) {
            Some(mat) => {
                *mat = data.clone();
            }
            None => {
                log::error!("Tried to update non-existing material {}", id);
                return;
            }
        };

        if let Some(sh_id) = &data.shader {
            if !self.shaders.contains_key(sh_id) {
                if let Err(e) = self.find_and_compile_shader(sh_id) {
                    log::error!("Could not find or compile shader: {}", e);
                }
            }
        }
    }
}

#[derive(Debug, Error)]
enum FindCompileShaderErr {
    #[error("Could not find shader with the given ID")]
    Find,

    #[error("Could not compile shader")]
    Compile(#[from] shader::CreateErr),
}

impl Window {
    fn find_and_compile_shader(&mut self, id: &ShaderId) -> Result<(), FindCompileShaderErr> {
        log::trace!("Finding and compiling shader with ID {}", id);

        debug_assert!(!self.shaders.contains_key(id));

        let shader_source = self.shader_resolver.find_set(id);

        if shader_source.is_none() {
            return Err(FindCompileShaderErr::Find);
        }

        let shader_source = shader_source.unwrap();

        let compiled = GlShaderProgram::new(&self.bindings, shader_source)?;

        self.shaders.insert(id.clone(), compiled);

        Ok(())
    }
}
