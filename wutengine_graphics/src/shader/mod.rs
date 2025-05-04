//! Shader abstractions and descriptions used for communication between the WutEngine runtime and graphics backends

use core::fmt::Display;
use core::hash::Hash;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

pub mod builtins;
mod compiled;
mod id;
mod raw;
mod resolver;
pub mod uniform;

use bitflags::bitflags;
use md5::{Digest, Md5};

pub use compiled::*;
pub use id::*;
pub use raw::*;
pub use resolver::*;

/// A struct representing a full shader program, including all pipeline stages
#[derive(Debug, Clone)]
pub enum Shader {
    /// A raw, uncompiled shader. Not yet usable by the backend
    Raw(RawShader),

    /// A compiled shader.
    Compiled(CompiledShader),
}

impl Shader {
    /// Returns the identifier of the shader, ignoring variants
    pub fn ident(&self) -> &String {
        match self {
            Shader::Raw(r) => &r.ident,
            Shader::Compiled(c) => c.id.ident(),
        }
    }

    pub fn id(&self) -> ShaderId {
        match self {
            Shader::Raw(raw_shader) => ShaderId::new_no_keywords(&raw_shader.ident),
            Shader::Compiled(compiled_shader) => compiled_shader.id.clone(),
        }
    }
}

/// Returns whether the given string would be a valid shader keyword identifier
pub fn is_valid_keyword(keyword: &str) -> bool {
    if keyword.is_empty() {
        return false;
    }

    let first_char_is_number = keyword.chars().next().unwrap().is_ascii_digit();
    let all_chars_valid = keyword
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');

    (!first_char_is_number) && all_chars_valid
}

/// A wrapper for the source code of the different stages of a [Shader]
#[derive(Debug, Clone)]
pub struct ShaderStages {
    /// The vertex pipeline stage
    pub vertex: Option<ShaderStage>,

    /// The fragment pipeline stage
    pub fragment: Option<ShaderStage>,
}

/// A shader stage
#[derive(Debug, Clone)]
pub struct ShaderStage {
    /// The source code
    pub source: String,

    /// The entry point of the stage
    pub entry: String,
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

/// The target platform of a shader
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderTarget {
    /// A shader compiled for OpenGL
    OpenGL,
}

impl Display for ShaderTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderTarget::OpenGL => write!(f, "OpenGL"),
        }
    }
}
