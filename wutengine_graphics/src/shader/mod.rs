use core::ops::RangeInclusive;
use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use wgpu::ShaderModule;
use wutengine_asset::{Asset, AssetHandle};

mod vertexlayout;

pub use vertexlayout::ShaderVertexLayout;

use crate::resource::GpuResource;

#[derive(Debug, Clone)]
pub enum Shader {
    Uncompiled {
        source: Arc<ShaderSource>,
        keywords: HashMap<String, i64>,
    },
    Compiled(Arc<CompiledShader>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderSource {
    pub name: String,
    pub code: String,
    pub available_keywords: HashMap<String, PossibleKeywordValue>,
    pub vertex_layout: ShaderVertexLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PossibleKeywordValue {
    Bool,
    Int(RangeInclusive<i64>),
    Uint(RangeInclusive<u64>),
}

impl Asset for ShaderSource {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledShader {
    pub name: String,
    pub keyword_hash: u128,
    pub source: naga::Module,

    #[serde(skip)]
    pub(crate) renderer_data: GpuResource<ShaderModule>,
}

impl Asset for CompiledShader {
    const FORCE_BINARY: bool = true;
}
