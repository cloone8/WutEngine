//! Module for an OpenGL window and associated context. Most of the main code of the backend is here.

use core::ffi::c_void;
use core::marker::PhantomData;
use core::ptr::{null, null_mut};
use core::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;
use std::rc::Rc;

use constantbuffers::ConstantBuffers;
use gl_from_raw_window_handle::{GlConfig, GlContext, Profile};
use glam::{Mat4, Vec4};
use thiserror::Error;
use wutengine_core::identifiers::WindowIdentifier;
use wutengine_graphics::color::Color;
use wutengine_graphics::material::{MaterialData, get_default_texture2d};
use wutengine_graphics::mesh::MeshData;
use wutengine_graphics::renderer::{
    DrawCall, HasDisplayHandle, HasWindowHandle, RendererMaterialId, RendererMeshId,
    RendererTexture2DId, Viewport,
};
use wutengine_graphics::shader::builtins::ShaderBuiltins;
use wutengine_graphics::shader::{Shader, ShaderResolver};
use wutengine_graphics::shader::{ShaderId, ShaderTarget};
use wutengine_graphics::texture::TextureData;

use crate::error::checkerr;
use crate::extensions::GlExtensions;
use crate::material::{self, GlMaterialBuffers};
use crate::mesh::{GlMeshBuffers, index_type_to_gl};
use crate::opengl::types::{GLchar, GLenum, GLint, GLsizei, GLsizeiptr, GLuint};
use crate::opengl::{self, Gl};
use crate::shader::uniform::const_blocks::{
    WutEngineInstanceConstants, WutEngineViewportConstants,
};
use crate::shader::{self, GlShaderProgram};
use crate::texture::tex2d::GlTexture2D;
use crate::vao::Vao;
use crate::{buffer, debug, extensions, texture};

mod constantbuffers;

static WINDOW_IDS: AtomicUsize = AtomicUsize::new(1);
static CURRENT_WINDOW_CONTEXT: AtomicUsize = AtomicUsize::new(0);

/// An OpenGL representation of a rendering window, with a unique context
/// and set of GPU resources
pub(crate) struct Window {
    id: usize,
    shader_resolver: Rc<dyn ShaderResolver>,
    context: GlContext,
    extensions: GlExtensions,
    bindings: Gl,

    // === Global buffers/constants/etc
    constant_buffers: ConstantBuffers,
    default_texture2d: GlTexture2D,

    // === Resources ===
    /// Shaders
    shaders: HashMap<ShaderId, GlShaderProgram>,

    /// Meshes
    meshes: HashMap<RendererMeshId, GlMeshBuffers>,

    /// Textures
    texture2ds: HashMap<RendererTexture2DId, GlTexture2D>,

    /// Materials
    materials: HashMap<RendererMaterialId, GlMaterialBuffers>,

    /// VAOs
    attributes: HashMap<RendererMeshId, Vao>,

    /// Trickery to ensure single-threaded rendering
    _not_send: PhantomData<*mut ()>,
}

#[profiling::all_functions]
impl Window {
    /// Creates a new window-specific context for the given native handle and initial size.
    /// Uses the provided shader resolver to find the shaders on disk.
    pub(crate) fn new(
        id: &WindowIdentifier,
        shader_resolver: Rc<dyn ShaderResolver>,
        handles: impl HasDisplayHandle + HasWindowHandle,
        size: (u32, u32),
    ) -> Self {
        let context = unsafe {
            profiling::scope!("Create Context");

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

        let window_id = WINDOW_IDS.fetch_add(1, Ordering::Relaxed);
        assert_ne!(0, window_id, "Window ID overflow"); // Should be exceedingly rare

        unsafe {
            profiling::scope!("Make Context Current");
            context.make_current();
            CURRENT_WINDOW_CONTEXT.store(window_id, Ordering::SeqCst);
        }

        let bindings = Gl::load_with(|s| context.get_proc_address(s));

        let extensions = GlExtensions::new(&bindings);

        log::info!("Found OpenGL extensions: {:#?}", extensions);
        unsafe {
            extensions::set_global(extensions);
        }

        if extensions.khr_debug {
            // TODO: Currently we leak this string if the window is destroyed
            let id_string_ptr = Box::into_raw(Box::new(id.to_string()));

            unsafe {
                if cfg!(debug_assertions) {
                    bindings.DebugMessageControl(
                        opengl::DONT_CARE,
                        opengl::DONT_CARE,
                        opengl::DONT_CARE,
                        0,
                        null(),
                        opengl::TRUE,
                    );
                } else {
                    // Release: Only high severity messages or error messages
                    bindings.DebugMessageControl(
                        opengl::DONT_CARE,
                        opengl::DONT_CARE,
                        opengl::DEBUG_SEVERITY_HIGH,
                        0,
                        null(),
                        opengl::TRUE,
                    );
                    bindings.DebugMessageControl(
                        opengl::DONT_CARE,
                        opengl::DEBUG_TYPE_ERROR,
                        opengl::DONT_CARE,
                        0,
                        null(),
                        opengl::TRUE,
                    );
                }

                bindings.DebugMessageCallback(
                    Some(debug::opengl_log_callback),
                    id_string_ptr as *mut c_void,
                );
            }
        }

        unsafe {
            bindings.Viewport(0, 0, size.0.try_into().unwrap(), size.1.try_into().unwrap());
            bindings.Enable(opengl::DEPTH_TEST);
        };

        checkerr!(&bindings);

        // We inject the default texture here
        let default_texture2d = GlTexture2D::new(&bindings).unwrap();
        unsafe {
            bindings.ActiveTexture(opengl::TEXTURE0);
            bindings.BindTexture(opengl::TEXTURE_2D, default_texture2d.handle().get());
        }

        default_texture2d.add_debug_label(&bindings, || Some("Default Texture2D"));

        texture::tex2d::upload_data_to_bound(&bindings, &get_default_texture2d::<16>());

        unsafe {
            bindings.BindTexture(opengl::TEXTURE_2D, 0);
        }

        let constant_buffers =
            ConstantBuffers::new(&bindings).expect("Could not create constant buffers");

        Self {
            id: window_id,
            shader_resolver,
            context,
            constant_buffers,
            bindings,
            extensions,
            default_texture2d,
            shaders: Default::default(),
            meshes: Default::default(),
            texture2ds: Default::default(),
            materials: Default::default(),
            attributes: Default::default(),
            _not_send: PhantomData,
        }
    }

    /// Makes the context of this window current, if it is not already
    fn ensure_context_current(&self) {
        let cur_context = CURRENT_WINDOW_CONTEXT.swap(self.id, Ordering::SeqCst);

        if cur_context != self.id {
            unsafe {
                self.context.make_current();
                extensions::set_global(self.extensions);
            }
        }
    }

    /// Destroys this [Window], its associated OpenGL context, and all resources
    pub(crate) fn destroy(mut self) {
        self.ensure_context_current();

        let mesh_ids: Vec<_> = self.meshes.keys().copied().collect();
        let material_ids: Vec<_> = self.materials.keys().copied().collect();
        let texture_ids: Vec<_> = self.texture2ds.keys().copied().collect();
        let shader_ids: Vec<_> = self.shaders.keys().cloned().collect();

        mesh_ids.into_iter().for_each(|id| self.delete_mesh(id));

        material_ids
            .into_iter()
            .for_each(|id| self.delete_material(id));

        texture_ids
            .into_iter()
            .for_each(|id| self.delete_texture2d(id));

        shader_ids.into_iter().for_each(|id| {
            let sh = self.shaders.remove(&id).unwrap();
            sh.destroy(&self.bindings);
        });

        self.default_texture2d.destroy(&self.bindings);
    }

    /// A function to be called whenever the size of the native window changed. Changes
    /// the OpenGL rendering viewport
    pub(crate) fn size_changed(&mut self, size: (u32, u32)) {
        self.ensure_context_current();

        unsafe {
            self.bindings
                .Viewport(0, 0, size.0 as GLint, size.1 as GLint);

            checkerr!(&self.bindings);
        };
    }

    /// Renders the given objects with the provided rendering context.
    /// The context holds the base view and projection matrices, as well as the
    /// viewport configuration. The objects represent the meshes to render, as well as which
    /// shaders and model matrices to use for rendering them.
    pub(crate) fn render(&mut self, viewport_context: &Viewport, objects: &[DrawCall]) {
        self.ensure_context_current();

        let gl = &self.bindings;

        let _gl_dbg_marker = debug::debug_marker_group(gl, || "Render Window");

        // First we transform the projection matrix depth range from the universal [0..1] to OpenGL [-1..1]
        const DEPTH_REMAP_MAT: Mat4 = Mat4::from_cols(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 2.0, 0.0),
            Vec4::new(0.0, 0.0, -1.0, 1.0),
        );

        let projection_mat = viewport_context.projection_mat * DEPTH_REMAP_MAT;

        let viewport_constants = WutEngineViewportConstants {
            view_mat: viewport_context.view_mat.into(),
            projection_mat: projection_mat.into(),
            view_projection_mat: (projection_mat * viewport_context.view_mat).into(),
        };

        let clear_color = viewport_context.clear_color;

        unsafe {
            profiling::scope!("Clear Viewport");
            gl.ClearColor(clear_color.r, clear_color.g, clear_color.b, clear_color.a);
            checkerr!(gl);
            gl.Clear(opengl::COLOR_BUFFER_BIT | opengl::DEPTH_BUFFER_BIT);
            checkerr!(gl);
        }

        // Update viewport constants
        unsafe {
            gl.BindBuffer(
                opengl::UNIFORM_BUFFER,
                self.constant_buffers.viewport_constants.handle().get(),
            );
            gl.BufferSubData(
                opengl::UNIFORM_BUFFER,
                0,
                size_of::<WutEngineViewportConstants>() as GLsizeiptr,
                &raw const viewport_constants as *const c_void,
            );
        }

        checkerr!(gl);

        {
            profiling::scope!("Render Objects");

            for object in objects {
                profiling::scope!("Render Object");
                let _gl_dbg_marker = debug::debug_marker_group(gl, || "Render object");

                let instance_constants = WutEngineInstanceConstants {
                    model_mat: object.object_to_world.into(),
                };

                // Update instance constants
                unsafe {
                    gl.BindBuffer(
                        opengl::UNIFORM_BUFFER,
                        self.constant_buffers.instance_constants.handle().get(),
                    );
                    gl.BufferSubData(
                        opengl::UNIFORM_BUFFER,
                        0,
                        size_of::<WutEngineInstanceConstants>() as GLsizeiptr,
                        &raw const instance_constants as *const c_void,
                    );
                }

                checkerr!(gl);

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

                let shader_id = material.target_shader.as_ref();
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
                unsafe {
                    gl.BindVertexArray(vao.handle().get());
                }
                if !vao.layout_matches(&mesh.vertex_layout, shader.get_vertex_layout()) {
                    unsafe {
                        gl.BindBuffer(opengl::ARRAY_BUFFER, mesh.vertex.handle().get());
                        gl.BindBuffer(opengl::ELEMENT_ARRAY_BUFFER, mesh.index.handle().get());
                    }

                    vao.set_layout(gl, &mesh.vertex_layout, shader.get_vertex_layout().clone());
                }

                unsafe {
                    gl.UseProgram(shader.handle().get());
                }

                // Set the constants
                for (&builtin, builtin_bindings) in &shader.builtins {
                    let handle = match builtin {
                        ShaderBuiltins::VIEWPORT_CONSTS => {
                            self.constant_buffers.viewport_constants.handle()
                        }
                        ShaderBuiltins::INSTANCE_CONSTS => {
                            self.constant_buffers.instance_constants.handle()
                        }
                        _ => unreachable!("Unknown builtin"),
                    };

                    for binding in builtin_bindings {
                        unsafe {
                            gl.BindBufferBase(
                                opengl::UNIFORM_BUFFER,
                                (binding.binding) as GLuint,
                                handle.get(),
                            );
                        }

                        checkerr!(gl);
                    }
                }

                // Set the uniforms
                let mut first_free_texture_unit = opengl::TEXTURE0;

                for (uniform_name, uniform) in &shader.uniforms {
                    let param_value = match material.parameter_values.get(uniform_name) {
                        Some(val) => val,
                        None => {
                            log::error!(
                                "Missing material parameter value for uniform {}",
                                uniform_name
                            );
                            continue;
                        }
                    };

                    match (uniform, param_value.value()) {
                        (
                            shader::uniform::GlShaderUniform::Sampler { location, .. },
                            material::GlMaterialUniformValue::Texture2D(texid),
                        ) => {
                            let texture = match self.texture2ds.get(texid) {
                                Some(t) => t,
                                None => {
                                    log::error!("Missing texture2d with ID {}", texid);
                                    continue;
                                }
                            };

                            unsafe {
                                gl.ActiveTexture(first_free_texture_unit);
                                gl.BindTexture(opengl::TEXTURE_2D, texture.handle().get());
                                gl.Uniform1i(*location, first_free_texture_unit as GLint);
                            }

                            checkerr!(gl);

                            first_free_texture_unit += 1;
                        }
                        (
                            shader::uniform::GlShaderUniform::Block { binding, .. },
                            material::GlMaterialUniformValue::Block(buffer),
                        ) => {
                            unsafe {
                                gl.BindBufferBase(
                                    opengl::UNIFORM_BUFFER,
                                    (*binding) as GLuint,
                                    buffer.handle().get(),
                                );
                            }

                            checkerr!(gl);
                        }
                        (other_uniform, other_value) => {
                            unreachable!(
                                "Uniform type and material value mismatch: {:#?}\n{:#?}",
                                other_uniform, other_value
                            );
                        }
                    }
                }

                // shader.set_uniform_defaults(
                //     gl,
                //     &material.parameters,
                //     &mut first_free_texture_unit,
                //     &mut self.default_texture,
                // );

                // shader.set_uniforms(
                //     gl,
                //     &material.parameters,
                //     &mut first_free_texture_unit,
                //     &mut self.textures,
                // );

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

                unsafe {
                    gl.BindVertexArray(0);
                }
            }
        }

        profiling::scope!("Swap Buffers");
        self.context.swap_buffers();
    }
}

/// Resource management
#[profiling::all_functions]
impl Window {
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
                if let Err(e) = self.find_and_compile_shader(sh_id) {
                    log::error!("Could not find or compile shader: {}", e);
                    return;
                }
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

#[profiling::all_functions]
impl Window {
    fn find_and_compile_shader(&mut self, id: &ShaderId) -> Result<(), FindCompileShaderErr> {
        log::trace!("Finding and compiling shader with ID {}", id);

        debug_assert!(!self.shaders.contains_key(id));

        let shader_source = self.shader_resolver.find_set(id);

        if shader_source.is_none() {
            return Err(FindCompileShaderErr::Find);
        }

        let shader_source = shader_source.unwrap();

        let compiled = match shader_source {
            Shader::Raw(raw) => {
                wutengine_shadercompiler::compile(raw, ShaderTarget::OpenGL, &HashMap::new())?
            }
            Shader::Compiled(compiled) => {
                assert_eq!(ShaderTarget::OpenGL, compiled.target);
                compiled.clone()
            }
        };

        log::info!("Compiled shader: {:#?}", compiled);

        debug_assert_eq!(ShaderTarget::OpenGL, compiled.target);

        let gl_compiled = GlShaderProgram::new(&self.bindings, &compiled)?;

        self.shaders.insert(id.clone(), gl_compiled);

        Ok(())
    }
}
