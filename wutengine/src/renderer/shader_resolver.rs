//! The various WutEngine builtin shader resolvers.

use std::collections::HashMap;
use wutengine_graphics::shader::resolver::ShaderResolver;
use wutengine_graphics::shader::{
    Shader, ShaderId, ShaderStages, ShaderVertexLayout, Uniform, UniformBinding, UniformType,
};

use crate::map;

/// The embedded shader resolver. Will use shaders from the [crate::embedded] module
/// only.
pub(crate) struct EmbeddedShaderResolver {
    sets: HashMap<ShaderId, Shader>,
}

impl EmbeddedShaderResolver {
    /// Creates a new [EmbeddedShaderResolver] with all default embedded shaders
    pub(crate) fn new() -> Self {
        let mut sets = HashMap::default();

        sets.insert(
            ShaderId::new("unlit"),
            Shader {
                id: ShaderId::new("unlit"),
                source: ShaderStages {
                    vertex: Some(
                        include_str!(concat!(
                            env!("CARGO_MANIFEST_DIR"),
                            "/shaders/unlit/vertex.glsl"
                        ))
                        .to_string(),
                    ),
                    fragment: Some(
                        include_str!(concat!(
                            env!("CARGO_MANIFEST_DIR"),
                            "/shaders/unlit/fragment.glsl"
                        ))
                        .to_string(),
                    ),
                },
                vertex_layout: ShaderVertexLayout {
                    position: Some(0),
                    ..Default::default()
                },
                uniforms: map![
                    "wuteng_ModelMat" => Uniform {
                        ty: UniformType::Mat4,
                        binding: UniformBinding {
                            name: "wuteng_ModelMat".to_string(),
                            group: 0,
                            binding: 0
                        }
                    },
                    "wuteng_ViewMat" => Uniform {
                        ty: UniformType::Mat4,
                        binding: UniformBinding {
                            name: "wuteng_ViewMat".to_string(),
                            group: 0,
                            binding: 0
                        }
                    },
                    "wuteng_ProjectionMat" => Uniform {
                        ty: UniformType::Mat4,
                        binding: UniformBinding {
                            name: "wuteng_ProjectionMat".to_string(),
                            group: 0,
                            binding: 0
                        }
                    },
                    "baseColor" => Uniform {
                        ty: UniformType::Vec4,
                        binding: UniformBinding {
                            name: "baseColor".to_string(),
                            group: 0,
                            binding: 0
                        }
                    }
                ],
            },
        );

        Self { sets }
    }
}

impl ShaderResolver for EmbeddedShaderResolver {
    fn find_set(&self, id: &ShaderId) -> Option<&Shader> {
        self.sets.get(id)
    }
}
