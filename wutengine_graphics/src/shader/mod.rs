use core::fmt::Display;
use core::hash::Hash;

use nohash_hasher::IsEnabled;

pub mod resolver;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
}

#[derive(Debug, Clone)]
pub struct ShaderSet {
    pub id: ShaderSetId,

    pub vertex_source: Option<String>,
    pub fragment_source: Option<String>,
}

impl ShaderSet {
    pub fn get_stage(&self, stage: ShaderStage) -> Option<&str> {
        match stage {
            ShaderStage::Vertex => self.vertex_source.as_deref(),
            ShaderStage::Fragment => self.fragment_source.as_deref(),
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShaderSetId(String);

impl ShaderSetId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl Display for ShaderSetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl IsEnabled for ShaderSetId {}
