use thiserror::Error;
use wutengine_graphics::material::MaterialData;
use wutengine_graphics::mesh::MeshData;
use wutengine_graphics::renderer::{RendererMaterialId, RendererMeshId, RendererTexture2DId};
use wutengine_graphics::shader::{Shader, ShaderResolver, ShaderTarget, ShaderVariantId};
use wutengine_graphics::texture::TextureData;

use crate::error::checkerr;
use crate::material::GlMaterialBuffers;
use crate::mesh::GlMeshBuffers;
use crate::opengl::Gl;
use crate::shader::{self, GlShaderProgram};
use crate::texture::tex2d::GlTexture2D;
use crate::vao::Vao;
use crate::{opengl, texture};

use super::Context;

/// Resource management
#[profiling::all_functions]
impl Context {
    /// Creates new OpenGL buffers for the given mesh and registers it with the
    /// context
    pub(crate) fn create_mesh(&mut self, id: RendererMeshId) {
        log::trace!("Creating mesh {}", id);

        self.ensure_context_current();

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

        unsafe {
            self.bindings.BindVertexArray(attributes.handle().get());
            self.bindings
                .BindBuffer(opengl::ARRAY_BUFFER, buffers.vertex.handle().get());
            self.bindings
                .BindBuffer(opengl::ELEMENT_ARRAY_BUFFER, buffers.index.handle().get());
        }

        checkerr!(&self.bindings);

        attributes.set_debug_label(&self.bindings, || Some(format!("{}:VAO", id)));

        buffers
            .vertex
            .set_debug_label(&self.bindings, || Some(format!("{}:vertex_buffer", id)));

        buffers
            .index
            .set_debug_label(&self.bindings, || Some(format!("{}:index_buffer", id)));

        self.meshes.insert(id, buffers);
        self.attributes.insert(id, attributes);
    }

    /// Deletes the GPU resources for the given mesh
    pub(crate) fn delete_mesh(&mut self, id: RendererMeshId) {
        log::trace!("Deleting mesh {}", id);

        self.ensure_context_current();

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

        self.ensure_context_current();

        let buffers = self.meshes.get_mut(&id);

        if buffers.is_none() {
            log::error!("Trying to update data of unknown mesh {}", id);
            return;
        }

        let buffers = buffers.unwrap();

        buffers.upload_data(&self.bindings, data);
    }

    /// Generates OpenGL buffers for the given texture and registers it with the context
    pub(crate) fn create_texture2d(&mut self, id: RendererTexture2DId) {
        log::trace!("Creating texture {}", id);

        self.ensure_context_current();

        debug_assert!(
            !self.texture2ds.contains_key(&id),
            "Texture ID already exists"
        );

        let texture = match GlTexture2D::new(&self.bindings) {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to create OpenGL texture: {:#?}", e);
                return;
            }
        };

        unsafe {
            self.bindings.ActiveTexture(opengl::TEXTURE0);
            self.bindings
                .BindTexture(opengl::TEXTURE_2D, texture.handle().get());
        }

        texture.add_debug_label(&self.bindings, || Some(format!("tex2d:{}", id)));

        self.texture2ds.insert(id, texture);
    }

    /// Deletes the given texture and its associated GPU resources in this context
    pub(crate) fn delete_texture2d(&mut self, id: RendererTexture2DId) {
        log::trace!("Deleting texture {}", id);

        self.ensure_context_current();

        let texture = self.texture2ds.remove(&id);

        match texture {
            Some(t) => t.destroy(&self.bindings),
            None => {
                log::warn!("Tried to delete non-existing texture {}", id)
            }
        };
    }

    /// Uploads new texture data for the given texture ID
    pub(crate) fn update_texture2d(&mut self, id: RendererTexture2DId, data: &TextureData) {
        log::trace!("Updating texture {}", id);

        self.ensure_context_current();

        let texture = self.texture2ds.get_mut(&id);

        if texture.is_none() {
            log::error!("Trying to update unknown texture {}", id);
            return;
        }

        let texture = texture.unwrap();

        // For consistency, just bind to the first texture unit
        unsafe {
            self.bindings.ActiveTexture(opengl::TEXTURE0);
            self.bindings
                .BindTexture(opengl::TEXTURE_2D, texture.handle().get());
        }

        texture::tex2d::upload_data_to_bound(&self.bindings, data);

        unsafe {
            self.bindings.BindTexture(opengl::TEXTURE_2D, 0);
        }
    }

    /// Registers the given material with this OpenGL window and context
    pub(crate) fn create_material(&mut self, id: RendererMaterialId) {
        log::trace!("Creating material {}", id);

        self.ensure_context_current();

        debug_assert!(
            !self.materials.contains_key(&id),
            "Material ID already exists"
        );

        self.materials.insert(id, GlMaterialBuffers::new());
    }

    /// Removes the given material from this OpenGL window and context
    pub(crate) fn delete_material(&mut self, id: RendererMaterialId) {
        log::trace!("Deleting material {}", id);

        self.ensure_context_current();

        let materialdata = self.materials.remove(&id);

        if let Some(materialdata) = materialdata {
            materialdata.destroy(&self.bindings);
        } else {
            log::warn!("Tried to delete non-existing material {}", id);
        }
    }

    /// Updates the given material
    pub(crate) fn update_material(&mut self, id: RendererMaterialId, data: &MaterialData) {
        log::trace!("Updating material {}", id);

        self.ensure_context_current();

        if let Some(sh_id) = &data.shader {
            if !self.shaders.contains_key(sh_id) {
                let compiled = match find_and_compile_shader(
                    &self.bindings,
                    self.shader_resolver.as_ref(),
                    sh_id,
                ) {
                    Ok(sh) => sh,
                    Err(e) => {
                        log::error!("Could not find or compile shader: {}", e);
                        return;
                    }
                };

                self.shaders.insert(sh_id.clone(), compiled);
            }
        } else {
            log::error!("Cannot update material {} without shader set", id);
            return;
        }

        let shaderprogram = self.shaders.get(data.shader.as_ref().unwrap()).unwrap();

        match self.materials.get_mut(&id) {
            Some(mat) => {
                mat.update(&self.bindings, &id, shaderprogram, data);
            }
            None => {
                log::error!("Tried to update non-existing material {}", id);
                return;
            }
        };
    }
}

#[derive(Debug, Error)]
enum FindCompileShaderErr {
    #[error("Could not find shader with the given ID")]
    Find,

    #[error("Could not cross-compile raw shader to OpenGL shader: {}", .0)]
    CrossCompile(#[from] wutengine_shadercompiler::CompileErr),

    #[error("Could not compile OpenGL shader: {}", .0)]
    GlCompile(#[from] shader::CreateErr),
}

#[profiling::function]
fn find_and_compile_shader(
    gl: &Gl,
    shader_resolver: &dyn ShaderResolver,
    id: &ShaderVariantId,
) -> Result<GlShaderProgram, FindCompileShaderErr> {
    log::trace!("Finding and compiling shader with ID {}", id);

    let shader_source = shader_resolver.find_set(id);

    if shader_source.is_none() {
        return Err(FindCompileShaderErr::Find);
    }

    let shader_source = shader_source.unwrap();

    let compiled = match shader_source {
        Shader::Raw(raw) => wutengine_shadercompiler::compile(
            raw,
            ShaderTarget::OpenGL,
            &id.keywords().into_iter().cloned().collect(),
        )?,
        Shader::Compiled(compiled) => {
            assert_eq!(ShaderTarget::OpenGL, compiled.target);
            compiled.clone()
        }
    };

    log::info!("Compiled shader: {:#?}", compiled);

    debug_assert_eq!(ShaderTarget::OpenGL, compiled.target);

    let gl_compiled = GlShaderProgram::new(gl, &compiled)?;

    Ok(gl_compiled)
}
