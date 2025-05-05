//! Material buffers and mapping

use core::ffi::c_void;
use std::collections::HashMap;

use wutengine_graphics::material::{MaterialData, MaterialParameter};
use wutengine_graphics::renderer::{RendererMaterialId, RendererTexture2DId};
use wutengine_graphics::shader::ShaderVariantId;

use crate::buffer::GlBuffer;
use crate::error::checkerr;
use crate::opengl::{self, Gl};
use crate::shader::GlShaderProgram;
use crate::shader::uniform::GlShaderUniform;
use crate::shader::uniform::std140::param_to_std140_buffer;

/// A set of buffers and metadata for a single WutEngine material,
/// and how it maps to OpenGL
#[derive(Debug)]
pub(crate) struct GlMaterialBuffers {
    /// The shader this material is currently configured for
    pub(crate) target_shader: Option<ShaderVariantId>,

    /// The underlying buffers assigned to each parameter
    pub(crate) parameter_values: HashMap<String, GlMaterialUniform>,
}

#[derive(Debug)]
pub(crate) struct GlMaterialUniform {
    value: GlMaterialUniformValue,
}

impl GlMaterialUniform {
    /// Getter for the value of a material uniform
    #[inline(always)]
    pub(crate) const fn value(&self) -> &GlMaterialUniformValue {
        &self.value
    }
}

/// The value for a single material uniform
#[derive(Debug)]
pub(crate) enum GlMaterialUniformValue {
    /// A uniform block. Contains the buffer with the block data
    Block(GlBuffer),

    /// A texture 2D (sampler). Contains the texture ID to be bound
    Texture2D(RendererTexture2DId),
}

#[profiling::all_functions]
impl GlMaterialBuffers {
    pub(crate) fn new() -> Self {
        Self {
            target_shader: None,
            parameter_values: HashMap::default(),
        }
    }

    pub(crate) fn update(
        &mut self,
        gl: &Gl,
        my_id: &RendererMaterialId,
        target_shader: &GlShaderProgram,
        data: &MaterialData,
    ) {
        self.target_shader = Some(
            data.shader
                .clone()
                .expect("Trying to update from material without target shader"),
        );

        for (uform_name, uform_data) in &target_shader.uniforms {
            if let Some(param) = data.parameters.get(uform_name) {
                self.update_single_uniform(gl, my_id, uform_name.clone(), uform_data, param)
            } else {
                log::warn!(
                    "Missing value for shader uniform {} in material. All params: {:#?}",
                    uform_name,
                    data.parameters
                );
                continue;
            }
        }
    }

    fn update_single_uniform(
        &mut self,
        gl: &Gl,
        my_id: &RendererMaterialId,
        name: String,
        uniform: &GlShaderUniform,
        param: &MaterialParameter,
    ) {
        match self.parameter_values.get_mut(&name) {
            Some(existing) => match &mut existing.value {
                GlMaterialUniformValue::Block(buffer) => {
                    let param_bytes = param_to_std140_buffer(param);

                    unsafe {
                        gl.BindBuffer(opengl::UNIFORM_BUFFER, buffer.handle().get());
                        gl.BufferSubData(
                            opengl::UNIFORM_BUFFER,
                            0,
                            param_bytes.len() as isize,
                            param_bytes.as_ptr() as *const c_void,
                        );
                    }

                    checkerr!(gl);
                }
                GlMaterialUniformValue::Texture2D(tex_id) => match param {
                    MaterialParameter::Texture2D(new_tex_id) => *tex_id = *new_tex_id,
                    _ => panic!(
                        "Unsupported parameter type for Sampler2D uniform: {:#?}",
                        param
                    ),
                },
            },
            None => {
                let uniform_value = match uniform {
                    GlShaderUniform::Sampler { .. } => match param {
                        MaterialParameter::Texture2D(renderer_texture2d_id) => {
                            GlMaterialUniformValue::Texture2D(*renderer_texture2d_id)
                        }
                        _ => panic!(
                            "Unsupported parameter type for Sampler2D uniform: {:#?}",
                            param
                        ),
                    },
                    GlShaderUniform::Block { ty, .. } => {
                        if ty != &param.get_type() {
                            panic!(
                                "Unsupported parameter type for {} uniform: {:#?}",
                                ty, param
                            );
                        }

                        let buffer = GlBuffer::new(gl).expect("Failed to create OpenGL buffer");

                        unsafe {
                            gl.BindBuffer(opengl::UNIFORM_BUFFER, buffer.handle().get());
                        }

                        buffer.set_debug_label(gl, || {
                            Some(format!(
                                "material:{}:{}.{}",
                                my_id,
                                self.target_shader.as_ref().unwrap(),
                                name
                            ))
                        });

                        let param_std140 = param_to_std140_buffer(param);
                        unsafe {
                            gl.BufferData(
                                opengl::UNIFORM_BUFFER,
                                param_std140.len() as isize,
                                param_std140.as_ptr() as *const c_void,
                                opengl::DYNAMIC_DRAW,
                            );
                        }

                        checkerr!(gl);

                        GlMaterialUniformValue::Block(buffer)
                    }
                };

                let uniform_data = GlMaterialUniform {
                    value: uniform_value,
                };

                self.parameter_values.insert(name, uniform_data);
            }
        }
    }

    pub(crate) fn destroy(self, gl: &Gl) {
        for (_name, uniform) in self.parameter_values {
            if let GlMaterialUniformValue::Block(buffer) = uniform.value {
                buffer.destroy(gl);
            }
        }
    }
}
