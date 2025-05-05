use core::ops::RangeInclusive;
use std::collections::{HashMap, HashSet};

use super::builtins::ShaderBuiltins;
use super::uniform::Uniform;
use super::{ShaderStages, ShaderVertexLayout};

/// A raw, uncompiled [super::Shader]. Not yet usable by the backend
#[derive(Debug, Clone)]
pub struct RawShader {
    /// The identifier of this shader
    pub ident: String,

    /// Which compile-time keywords can be set on this shader
    pub available_keywords: HashMap<String, ValidKeywordValues>,

    /// The source code of the shader and its stages
    pub source: ShaderStages,

    /// The expected layout of the vertex buffer
    pub vertex_layout: ShaderVertexLayout,

    /// Which builtins the shader uses
    pub builtins: ShaderBuiltins,

    /// The different uniforms
    pub uniforms: HashMap<String, Uniform>,
}

/// A range of valid values for a shader keyword
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidKeywordValues {
    /// Either 0 or 1
    Bool,

    /// A range of values
    Range(RangeInclusive<u32>),

    /// A set of valid values
    Set(HashSet<u32>),
}

impl ValidKeywordValues {
    /// Returns whether the given value is valid for this set
    pub fn is_valid(&self, val: u32) -> bool {
        match self {
            ValidKeywordValues::Bool => val == 0 || val == 1,
            ValidKeywordValues::Range(r) => r.contains(&val),
            ValidKeywordValues::Set(s) => s.contains(&val),
        }
    }
}
