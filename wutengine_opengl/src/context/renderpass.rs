use core::ffi::c_void;
use core::ptr::null_mut;
use std::collections::HashMap;

use glam::{Mat4, Vec4};
use wutengine_graphics::renderer::{DrawCall, RendererTexture2DId, Viewport};
use wutengine_graphics::shader::builtins::ShaderBuiltins;

use crate::constantbuffers::ConstantBuffers;
use crate::error::checkerr;
use crate::material::GlMaterialBuffers;
use crate::mesh::index_type_to_gl;
use crate::opengl::Gl;
use crate::opengl::types::{GLint, GLsizeiptr, GLuint};
use crate::shader::GlShaderProgram;
use crate::shader::uniform::const_blocks::{
    WutEngineInstanceConstants, WutEngineViewportConstants,
};
use crate::texture::tex2d::GlTexture2D;
use crate::{debug, material, opengl, shader};

use super::Context;

#[profiling::all_functions]
impl Context {
    /// Renders the given objects with the provided rendering context.
    /// The context holds the base view and projection matrices, as well as the
    /// viewport configuration. The objects represent the meshes to render, as well as which
    /// shaders and model matrices to use for rendering them.
    pub(crate) fn render(&mut self, viewport_context: &Viewport, objects: &[DrawCall]) {
        self.ensure_context_current();

        let gl = &self.bindings;

        let _gl_dbg_marker = debug::debug_marker_group(gl, || "Render Window");

        // MacOS seems to be late with creating the framebuffer

        unsafe {
            if cfg!(target_os = "macos")
                && gl.CheckFramebufferStatus(opengl::FRAMEBUFFER) != opengl::FRAMEBUFFER_COMPLETE
            {
                return;
            }
        }

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

                    checkerr!(gl);
                    vao.set_layout(gl, &mesh.vertex_layout, shader.get_vertex_layout().clone());
                }

                unsafe {
                    gl.UseProgram(shader.handle().get());
                }
                checkerr!(gl);

                // Set the constants
                bind_constants(gl, shader, &self.constant_buffers);

                // Set the uniforms
                bind_uniforms(gl, shader, material, &self.texture2ds);

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
                checkerr!(gl);
            }
        }

        profiling::scope!("Swap Buffers");
        self.gl_context.swap_buffers();
    }
}

#[profiling::function]
fn bind_constants(gl: &Gl, shader: &GlShaderProgram, constant_buffers: &ConstantBuffers) {
    for (&builtin, builtin_bindings) in &shader.builtins {
        let handle = match builtin {
            ShaderBuiltins::VIEWPORT_CONSTS => constant_buffers.viewport_constants.handle(),
            ShaderBuiltins::INSTANCE_CONSTS => constant_buffers.instance_constants.handle(),
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
}

#[profiling::function]
fn bind_uniforms(
    gl: &Gl,
    shader: &GlShaderProgram,
    material: &GlMaterialBuffers,
    texture2ds: &HashMap<RendererTexture2DId, GlTexture2D>,
) {
    let mut first_free_texture_unit = 0;

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
                let texture = match texture2ds.get(texid) {
                    Some(t) => t,
                    None => {
                        log::error!("Missing texture2d with ID {}", texid);
                        continue;
                    }
                };

                unsafe {
                    gl.ActiveTexture(opengl::TEXTURE0 + first_free_texture_unit);
                    checkerr!(gl);
                    gl.BindTexture(opengl::TEXTURE_2D, texture.handle().get());
                    checkerr!(gl);
                    gl.Uniform1i(*location, first_free_texture_unit as GLint);
                    checkerr!(gl);
                }

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

    if first_free_texture_unit > 0 {
        log::trace!("Bound {} texture units", first_free_texture_unit);
    }
}
