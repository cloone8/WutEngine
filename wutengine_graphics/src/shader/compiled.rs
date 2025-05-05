use std::collections::HashMap;

use super::builtins::ShaderBuiltins;
use super::uniform::{SingleUniformBinding, Uniform};
use super::{ShaderVariantId, ShaderStages, ShaderTarget, ShaderVertexLayout};

/// A compiled [super::Shader].
#[derive(Debug, Clone)]
pub struct CompiledShader {
    /// The variant ID of this compiled shader
    pub id: ShaderVariantId,

    /// The target backend for this compiled shader
    pub target: ShaderTarget,

    pub target_meta: ShaderTargetMeta,

    /// The source code of the shader and its stages
    pub source: ShaderStages,

    /// The expected layout of the vertex buffer
    pub vertex_layout: ShaderVertexLayout,

    /// Which builtins the shader uses
    pub builtins: ShaderBuiltins,

    /// The different uniforms
    pub uniforms: HashMap<String, Uniform>,
}

#[derive(Debug, Clone)]
pub enum ShaderTargetMeta {
    OpenGL(GLShaderMeta),
}

impl ShaderTargetMeta {
    pub fn as_opengl(&self) -> Option<&GLShaderMeta> {
        match self {
            Self::OpenGL(shmeta) => Some(shmeta),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GLShaderMeta {
    pub builtins_vertex: HashMap<ShaderBuiltins, SingleUniformBinding>,
    pub builtins_fragment: HashMap<ShaderBuiltins, SingleUniformBinding>,
}
