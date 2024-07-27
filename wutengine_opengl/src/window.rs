use std::collections::HashMap;

use gl_from_raw_window_handle::{GlConfig, GlContext, Profile};
use nohash_hasher::IntMap;
use wutengine_graphics::mesh::MeshData;
use wutengine_graphics::shader::{ShaderSource, ShaderVariant};
use wutengine_graphics::{
    renderer::{RenderContext, Renderable},
    windowing::{HasDisplayHandle, HasWindowHandle},
};

use crate::opengl::{self, Gl};
use crate::shader::load_builtin;
use crate::shaderprogram::ShaderProgram;
use crate::vao::Vao;
use crate::vbo::Vbo;

pub struct Window {
    context: GlContext,
    bindings: Gl,
    shaders: HashMap<ShaderVariant, ShaderProgram>,
    meshes: IntMap<usize, Vao>,
}

impl Window {
    pub fn new(handles: impl HasDisplayHandle + HasWindowHandle, size: (u32, u32)) -> Self {
        let context = unsafe {
            GlContext::create(
                &handles,
                GlConfig {
                    version: (3, 3),
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

        unsafe { bindings.Viewport(0, 0, size.0.try_into().unwrap(), size.1.try_into().unwrap()) };

        Self {
            context,
            bindings,
            shaders: HashMap::new(),
            meshes: IntMap::default(),
        }
    }

    pub fn size_changed(&mut self, size: (u32, u32)) {
        unsafe {
            self.context.make_current();
            self.bindings
                .Viewport(0, 0, size.0.try_into().unwrap(), size.1.try_into().unwrap());
        };
    }

    fn get_or_insert_shader(&mut self, shader: &ShaderVariant) -> &mut ShaderProgram {
        if self.shaders.contains_key(shader) {
            return self.shaders.get_mut(shader).unwrap();
        }

        log::info!("Unknown shader variant, loading from source");

        let sources = match shader.source {
            ShaderSource::Builtin { identifier } => load_builtin(&self.bindings, identifier),
        };

        if sources.is_none() {
            panic!("Cannot map shader variant to sources: {:#?}", shader);
        }

        let sources = sources.unwrap();

        if sources.is_err() {
            panic!(
                "Could not create shader sources: {:#?}",
                sources.unwrap_err()
            );
        }

        let sources = sources.unwrap();

        log::debug!("Loaded shaderset: {:#?}", sources);

        let program = ShaderProgram::new(&self.bindings, sources);

        if program.is_err() {
            panic!(
                "Could not create shader program: {:#?}",
                program.unwrap_err()
            );
        }

        let program = program.unwrap();

        self.shaders.insert(shader.clone(), program);

        self.shaders.get_mut(shader).unwrap()
    }

    fn get_or_insert_meshdata(&mut self, meshdata: &MeshData) -> &mut Vao {
        if self.meshes.contains_key(&meshdata.get_id()) {
            return self.meshes.get_mut(&meshdata.get_id()).unwrap();
        }

        log::info!("Unknown mesh, loading into buffers");
        let gl = &self.bindings;

        let mut vao = Vao::new(gl).unwrap();
        vao.bind(gl);

        let mut vbo = Vbo::new(gl).unwrap();
        vbo.bind(gl);
        vbo.buffer_data(gl, &meshdata.vertices);

        vao.set_vertex_attrs_for(gl, meshdata);

        self.meshes.insert(meshdata.get_id(), vao);

        self.meshes.get_mut(&meshdata.get_id()).unwrap()
    }

    pub fn render(&mut self, render_context: RenderContext, objects: &[Renderable]) {
        unsafe {
            self.context.make_current();
        }

        let gl = &self.bindings.clone();

        let clear_color = render_context.clear_color;

        unsafe {
            gl.ClearColor(clear_color.r, clear_color.g, clear_color.b, clear_color.a);
            gl.Clear(opengl::COLOR_BUFFER_BIT);
        }

        for object in objects {
            let vao = self.get_or_insert_meshdata(&object.mesh);
            vao.bind(gl);

            let program = self.get_or_insert_shader(&object.material.shader);
            program.use_program(gl).unwrap();

            unsafe {
                gl.DrawArrays(opengl::TRIANGLES, 0, 3);
            }
        }

        self.context.swap_buffers();
    }
}
