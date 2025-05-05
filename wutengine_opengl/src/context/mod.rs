//! Module for an OpenGL window and associated context. Most of the main code of the backend is here.

use core::ffi::c_void;
use core::marker::PhantomData;
use core::ptr::null;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;
use std::rc::Rc;

use gl_from_raw_window_handle::{GlConfig, GlContext, Profile};
use wutengine_core::identifiers::WindowIdentifier;
use wutengine_graphics::material::get_default_texture2d;
use wutengine_graphics::renderer::{
    HasDisplayHandle, HasWindowHandle, RendererMaterialId, RendererMeshId, RendererTexture2DId,
};
use wutengine_graphics::shader::ShaderResolver;
use wutengine_graphics::shader::ShaderVariantId;

use crate::constantbuffers::ConstantBuffers;
use crate::error::checkerr;
use crate::extensions::GlExtensions;
use crate::material::GlMaterialBuffers;
use crate::mesh::GlMeshBuffers;
use crate::opengl::types::GLint;
use crate::opengl::{self, Gl};
use crate::shader::GlShaderProgram;
use crate::texture::tex2d::GlTexture2D;
use crate::vao::Vao;
use crate::{debug, extensions, texture};

mod renderpass;
mod resources;

static WINDOW_IDS: AtomicUsize = AtomicUsize::new(1);
static CURRENT_WINDOW_CONTEXT: AtomicUsize = AtomicUsize::new(0);

/// An OpenGL representation of a rendering window, with a unique context
/// and set of GPU resources
pub(crate) struct Context {
    id: usize,
    shader_resolver: Rc<dyn ShaderResolver>,
    gl_context: GlContext,
    extensions: GlExtensions,
    bindings: Gl,

    // === Global buffers/constants/etc
    constant_buffers: ConstantBuffers,
    default_texture2d: GlTexture2D,

    // === Resources ===
    /// Shaders
    shaders: HashMap<ShaderVariantId, GlShaderProgram>,

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
impl Context {
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
            gl_context: context,
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
                self.gl_context.make_current();
                extensions::set_global(self.extensions);
            }
            checkerr!(&self.bindings);
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
        self.constant_buffers.destroy(&self.bindings)
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
}
