//! The various WutEngine builtin shader resolvers.

use std::collections::HashMap;
use wutengine_graphics::shader::builtins::ShaderBuiltins;
use wutengine_graphics::shader::uniform::{
    SingleUniformBinding, Uniform, UniformBinding, UniformType,
};
use wutengine_graphics::shader::{
    RawShader, Shader, ShaderId, ShaderStages, ShaderVertexLayout, ValidKeywordValues,
};
use wutengine_graphics::shader::{ShaderResolver, ShaderStage};

use crate::map;

/// The embedded shader resolver. Will use shaders from the [crate::embedded] module
/// only.
pub(crate) struct InMemoryShaderResolver {
    sets: HashMap<ShaderId, Shader>,
}

impl InMemoryShaderResolver {
    /// Creates a new [InMemoryShaderResolver] with all default embedded shaders
    pub(crate) fn new_embedded() -> Self {
        let mut sets = HashMap::default();

        sets.insert(
            ShaderId::new_no_keywords("unlit"),
            Shader::Raw(RawShader {
                ident: "unlit".to_string(),
                available_keywords: map![
                    "HAS_COLOR_MAP" => ValidKeywordValues::Bool
                ],
                builtins: ShaderBuiltins::INSTANCE_CONSTS | ShaderBuiltins::VIEWPORT_CONSTS,
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
                    }
                ],
            }),
        );

        Self { sets }
    }
}

impl ShaderResolver for InMemoryShaderResolver {
    fn find_set(&self, id: &ShaderId) -> Option<&Shader> {
        self.sets.get(id)
    }
}
