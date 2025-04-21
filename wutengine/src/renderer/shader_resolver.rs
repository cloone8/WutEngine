//! The various WutEngine builtin shader resolvers.

use std::collections::HashMap;
use wutengine_graphics::shader::ShaderTarget::{self};
use wutengine_graphics::shader::{
    Shader, ShaderId, ShaderStages, ShaderVertexLayout, SingleUniformBinding, Uniform,
    UniformBinding, UniformType,
};
use wutengine_graphics::shader::{ShaderResolver, ShaderStage};

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
                target: ShaderTarget::Raw,
                source: ShaderStages {
                    vertex: Some(ShaderStage {
                        source: include_str!(concat!(
                            env!("CARGO_MANIFEST_DIR"),
                            "/shaders/unlit/vertex.wgsl"
                        ))
                        .to_string(),
                        entry: "vertex_main".to_string(),
                    }),
                    fragment: Some(ShaderStage {
                        source: include_str!(concat!(
                            env!("CARGO_MANIFEST_DIR"),
                            "/shaders/unlit/fragment.wgsl"
                        ))
                        .to_string(),
                        entry: "fragment_main".to_string(),
                    }),
                },
                vertex_layout: ShaderVertexLayout {
                    position: Some(0),
                    uv: Some(1),
                    ..Default::default()
                },
                uniforms: map![
                    "wuteng_model_mat" => Uniform {
                        ty: UniformType::Array(Box::new(UniformType::Struct(map![
                            "model_mat" => UniformType::Mat4
                        ])), 5),
                        binding: UniformBinding::Standard(SingleUniformBinding {
                            name: "wuteng_model_mat".to_string(),
                            group: 0,
                            binding: 1
                        }),
                    },
                    "wuteng_vp" => Uniform {
                        ty: UniformType::Struct(map![
                            "inner" => UniformType::Struct(
                                map![
                                    "view" => UniformType::Mat4,
                                    "projection" => UniformType::Mat4
                                ]
                            ),
                            "test" => UniformType::Array(Box::new(UniformType::Vec3), 7)
                        ]),
                        binding: UniformBinding::Standard(SingleUniformBinding {
                            name: "wuteng_vp".to_string(),
                            group: 0,
                            binding: 0
                        }),
                    },
                    "base_color" => Uniform {
                        ty: UniformType::Vec4,
                        binding: UniformBinding::Standard(SingleUniformBinding {
                            name: "base_color".to_string(),
                            group: 1,
                            binding: 0
                        }),
                    },
                    "color_map" => Uniform {
                        ty: UniformType::Tex2D,
                        binding: UniformBinding::Texture {
                            texture: Some(SingleUniformBinding {
                                name: "color_map".to_string(),
                                group: 1,
                                binding: 2
                            }),
                            sampler: Some(SingleUniformBinding {
                                name: "color_map_tex".to_string(),
                                group: 1,
                                binding: 1
                            })
                        },
                    },
                    "has_color_map" => Uniform {
                        ty: UniformType::Uint32,
                        binding: UniformBinding::Standard(SingleUniformBinding {
                            name: "has_color_map".to_string(),
                            group: 1,
                            binding: 3
                        }),
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
