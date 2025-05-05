use std::collections::HashMap;

use super::builtins::ShaderBuiltins;
use super::uniform::{SingleUniformBinding, Uniform};
use super::{ShaderStages, ShaderTarget, ShaderVariantId, ShaderVertexLayout};

/// A compiled [super::Shader].
#[derive(Debug, Clone)]
pub struct CompiledShader {
    /// The variant ID of this compiled shader
    pub id: ShaderVariantId,

    /// The target backend for this compiled shader
    pub target: ShaderTarget,

    /// Per-target metadata
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

/// Per-target metadata
#[derive(Debug, Clone)]
pub enum ShaderTargetMeta {
    /// OpenGL shader metadata
    OpenGL(GLShaderMeta),
}

impl ShaderTargetMeta {
    /// Returns this metadata as its OpenGL variant, or [None] if the metadata
    /// is for another target
    pub fn as_opengl(&self) -> Option<&GLShaderMeta> {
        match self {
            Self::OpenGL(shmeta) => Some(shmeta),
        }
    }
}

/// OpenGL shader metadata.
#[derive(Debug, Clone)]
pub struct GLShaderMeta {
    /// Binding points for the builtins in the vertex shader stage
    pub builtins_vertex: HashMap<ShaderBuiltins, SingleUniformBinding>,

    /// Binding points for the builtins in the fragment shader stage
    pub builtins_fragment: HashMap<ShaderBuiltins, SingleUniformBinding>,
}
