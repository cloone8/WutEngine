use core::ptr::null;
use std::collections::HashMap;
use std::rc::Rc;

use gl_from_raw_window_handle::{GlConfig, GlContext, Profile};
use nohash_hasher::IntMap;
use wutengine_graphics::material::MaterialParameter;
use wutengine_graphics::mesh::{IndexBuffer, MeshData, MeshDataId};
use wutengine_graphics::renderer::{HasDisplayHandle, HasWindowHandle, Renderable, Viewport};
use wutengine_graphics::shader::resolver::ShaderResolver;
use wutengine_graphics::shader::uniforms::SharedShaderUniform;
use wutengine_graphics::shader::ShaderSetId;

use crate::error::check_gl_err;
use crate::mesh::GlMeshBuffers;
use crate::opengl::{self, Gl};
use crate::shader::program::ShaderProgram;
use crate::shader::set::GlShaderSet;
use crate::vao::Vao;

pub(crate) struct Window {
    shader_resolver: Rc<dyn ShaderResolver>,
    context: GlContext,
    bindings: Gl,
    shaders: HashMap<ShaderSetId, ShaderProgram>,
    meshes: IntMap<MeshDataId, GlMeshBuffers>,
    attributes: HashMap<(MeshDataId, ShaderSetId), Vao>,
}

impl Window {
    pub(crate) fn new(
        shader_resolver: Rc<dyn ShaderResolver>,
        handles: impl HasDisplayHandle + HasWindowHandle,
        size: (u32, u32),
    ) -> Self {
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

        check_gl_err!(bindings);

        Self {
            shader_resolver,
            context,
            bindings,
            shaders: HashMap::default(),
            meshes: IntMap::default(),
            attributes: HashMap::default(),
        }
    }

    pub(crate) fn size_changed(&mut self, size: (u32, u32)) {
        unsafe {
            self.context.make_current();
            self.bindings
                .Viewport(0, 0, size.0.try_into().unwrap(), size.1.try_into().unwrap());
            check_gl_err!(self.bindings);
        };
    }

    fn get_or_insert_shader<'a>(
        resolver: &dyn ShaderResolver,
        gl: &Gl,
        shaders: &'a mut HashMap<ShaderSetId, ShaderProgram>,
        shader: &ShaderSetId,
    ) -> &'a mut ShaderProgram {
        if shaders.contains_key(shader) {
            return shaders.get_mut(shader).unwrap();
        }

        log::info!("Unknown shader variant, loading from source");

        let sources = resolver.find_set(shader);

        if sources.is_none() {
            panic!("Cannot map shader variant to sources: {:#?}", shader);
        }

        let sources = sources.unwrap();
        let sources = GlShaderSet::from_sources(gl, &sources);

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
        meshes: &'a mut IntMap<MeshDataId, GlMeshBuffers>,
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

    pub(crate) fn get_object_data(
        &mut self,
        object: &Renderable,
    ) -> (&mut Vao, &mut ShaderProgram) {
        let mesh = &object.mesh;
        let shader = object.material.shader.clone();
        let vao_key = (mesh.get_id(), shader.clone());

        let program = Self::get_or_insert_shader(
            self.shader_resolver.as_ref(),
            &self.bindings,
            &mut self.shaders,
            &shader,
        );
        program.ensure_linked(&self.bindings).unwrap();

        if self.attributes.contains_key(&vao_key) {
            return (self.attributes.get_mut(&vao_key).unwrap(), program);
        }

        log::info!("Unknown mesh/shader combination. Creating VAO");

        let mesh_buffers = Self::get_or_insert_mesh_buffer(&self.bindings, &mut self.meshes, mesh);
        let gl = &self.bindings;

        let mut vao = Vao::new(gl).unwrap();

        vao.bind(gl);

        mesh_buffers.vertex.bind(gl);
        mesh_buffers.index.bind(gl);
        vao.set_vertex_attrs_for(gl, mesh_buffers, program);

        vao.unbind(gl);

        self.attributes.insert(vao_key.clone(), vao);

        (self.attributes.get_mut(&vao_key).unwrap(), program)
    }

    /// Renders the given objects with the provided rendering context.
    /// The context holds the base view and projection matrices, as well as the
    /// viewport configuration. The objects represent the meshes to render, as well as which
    /// shaders and model matrices to use for rendering them.
    pub(crate) fn render(&mut self, viewport_context: &Viewport, objects: &[Renderable]) {
        unsafe {
            self.context.make_current();
        }

        let gl = &self.bindings.clone();

        let clear_color = viewport_context.clear_color;

        unsafe {
            gl.ClearColor(clear_color.r, clear_color.g, clear_color.b, clear_color.a);
            gl.Clear(opengl::COLOR_BUFFER_BIT);
        }

        for object in objects {
            let (vao, program) = self.get_object_data(object);

            vao.bind(gl);
            program.use_program(gl).unwrap();

            unsafe {
                program
                    .set_uniform(
                        gl,
                        SharedShaderUniform::ModelMat.as_str(),
                        &MaterialParameter::Mat4(object.object_to_world),
                    )
                    .expect("Could not set model matrix");

                program
                    .set_uniform(
                        gl,
                        SharedShaderUniform::ViewMat.as_str(),
                        &MaterialParameter::Mat4(viewport_context.view_mat),
                    )
                    .expect("Could not set view matrix");

                program
                    .set_uniform(
                        gl,
                        SharedShaderUniform::ProjectionMat.as_str(),
                        &MaterialParameter::Mat4(viewport_context.projection_mat),
                    )
                    .expect("Could not set projection matrix");

                program
                    .set_uniforms(gl, &object.material.parameters)
                    .unwrap();
            }

            unsafe {
                //TODO: Dirty hack until I can get the amount of triangles properly from the VAO or soemthing
                gl.DrawElements(
                    opengl::TRIANGLES,
                    match &object.mesh.indices {
                        IndexBuffer::U16(vec) => vec.len().try_into().unwrap(),
                        IndexBuffer::U32(vec) => vec.len().try_into().unwrap(),
                    },
                    opengl::UNSIGNED_INT,
                    null(),
                );
            }
        }

        self.context.swap_buffers();
    }
}
