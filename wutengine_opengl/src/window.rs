use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use gl_from_raw_window_handle::{GlConfig, GlContext, Profile};
use nohash_hasher::IntMap;
use wutengine_graphics::mesh::MeshData;
use wutengine_graphics::shader::{ShaderSource, ShaderVariant};
use wutengine_graphics::{
    renderer::{RenderContext, Renderable},
    windowing::{HasDisplayHandle, HasWindowHandle},
};

use crate::gltypes::GlMeshBuffers;
use crate::opengl::{self, Gl};
use crate::shader::load_builtin;
use crate::shaderprogram::ShaderProgram;
use crate::vao::Vao;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderableKey<'a> {
    pub mesh_id: usize,
    pub shader: &'a ShaderVariant,
}

impl<'a> RenderableKey<'a> {
    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);

        hasher.finish()
    }
}

pub struct Window {
    context: GlContext,
    bindings: Gl,
    shaders: HashMap<ShaderVariant, ShaderProgram>,
    meshes: IntMap<usize, GlMeshBuffers>,
    attributes: IntMap<u64, Vao>,
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
            attributes: IntMap::default(),
        }
    }

    pub fn size_changed(&mut self, size: (u32, u32)) {
        unsafe {
            self.context.make_current();
            self.bindings
                .Viewport(0, 0, size.0.try_into().unwrap(), size.1.try_into().unwrap());
        };
    }

    fn get_or_insert_shader<'a>(
        gl: &Gl,
        shaders: &'a mut HashMap<ShaderVariant, ShaderProgram>,
        shader: &ShaderVariant,
    ) -> &'a mut ShaderProgram {
        if shaders.contains_key(shader) {
            return shaders.get_mut(shader).unwrap();
        }

        log::info!("Unknown shader variant, loading from source");

        let sources = match shader.source {
            ShaderSource::Builtin { identifier } => load_builtin(gl, identifier),
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

        let program = ShaderProgram::new(gl, sources);

        if program.is_err() {
            panic!(
                "Could not create shader program: {:#?}",
                program.unwrap_err()
            );
        }

        let program = program.unwrap();

        shaders.insert(shader.clone(), program);

        shaders.get_mut(shader).unwrap()
    }

    fn get_or_insert_mesh_buffer<'a>(
        gl: &Gl,
        meshes: &'a mut IntMap<usize, GlMeshBuffers>,
        mesh: &MeshData,
    ) -> &'a mut GlMeshBuffers {
        if meshes.contains_key(&mesh.get_id()) {
            return meshes.get_mut(&mesh.get_id()).unwrap();
        }

        log::info!("Unknown mesh, creating buffer");

        let buffers = GlMeshBuffers::new(gl, mesh).unwrap();

        meshes.insert(mesh.get_id(), buffers);

        meshes.get_mut(&mesh.get_id()).unwrap()
    }

    pub fn get_object_data(&mut self, object: &Renderable) -> (&mut Vao, &mut ShaderProgram) {
        let mesh = &object.mesh;
        let shader = &object.material.shader;

        let program = Self::get_or_insert_shader(&self.bindings, &mut self.shaders, shader);
        program.ensure_linked(&self.bindings).unwrap();

        let key = RenderableKey {
            mesh_id: mesh.get_id(),
            shader,
        }
        .get_hash();

        if self.attributes.contains_key(&key) {
            return (self.attributes.get_mut(&key).unwrap(), program);
        }

        log::info!("Unknown mesh/shader combination. Creating VAO");

        let mesh_buffers = Self::get_or_insert_mesh_buffer(&self.bindings, &mut self.meshes, mesh);
        let gl = &self.bindings;

        let mut vao = Vao::new(gl).unwrap();

        vao.bind(gl);

        mesh_buffers.vertex.bind(gl);
        vao.set_vertex_attrs_for(gl, mesh_buffers, program);

        vao.unbind(gl);

        self.attributes.insert(key, vao);

        (self.attributes.get_mut(&key).unwrap(), program)
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
            let (vao, program) = self.get_object_data(object);

            vao.bind(gl);
            program.use_program(gl).unwrap();

            unsafe {
                gl.DrawArrays(opengl::TRIANGLES, 0, 3);
            }
        }

        self.context.swap_buffers();
    }
}
