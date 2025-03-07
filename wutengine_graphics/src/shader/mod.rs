use core::fmt::Display;
use core::hash::Hash;
use std::collections::HashMap;

use nohash_hasher::IsEnabled;

pub mod attributes;
pub mod resolver;
pub mod uniforms;

/// The ID of a [Shader]
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShaderId(String);

impl ShaderId {
    /// Creates a new [ShaderId] from the given value
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl Display for ShaderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl IsEnabled for ShaderId {}

/// A struct representing a full shader program, including all pipeline stages
#[derive(Debug, Clone)]
pub struct Shader {
    /// The ID of this shader
    pub id: ShaderId,

    /// The source code of the shader and its stages
    pub source: ShaderStages,

    /// The expected layout of the vertex buffer
    pub vertex_layout: ShaderVertexLayout,

    /// The different uniforms
    pub uniforms: HashMap<String, Uniform>,
}

/// A wrapper for the source code of the different stages of a [Shader]
#[derive(Debug, Clone)]
pub struct ShaderStages {
    /// The vertex pipeline stage source
    pub vertex: Option<String>,

    /// The fragment pipeline stage source
    pub fragment: Option<String>,
}

impl ShaderStages {
    /// Returns true if any of the stages have source code
    pub fn has_any(&self) -> bool {
        self.vertex.is_some() || self.fragment.is_some()
    }
}

/// The expected layout for a single vertex, for a given [Shader]
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct ShaderVertexLayout {
    /// The location of the position attribute, if any
    pub position: Option<usize>,

    /// The location of the color attribute, if any
    pub color: Option<usize>,

    /// The location of the normal attribute, if any
    pub normal: Option<usize>,

    /// The location of the UV (texture coordinate) attribute, if any
    pub uv: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Uniform {
    pub ty: UniformType,
    pub binding: UniformBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UniformType {
    Vec3,
    Vec4,
    Mat4,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UniformBinding {
    pub name: String,
    pub group: usize,
    pub binding: usize,
}
