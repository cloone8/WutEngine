//! GPU Shaders

use alloc::sync::Arc;
use core::str::FromStr;

use wutengine_assets::FromSerializedAsset;
use wutengine_assets::assets::shader::SerializedShader;
use wutengine_shadercompiler::ShaderInfo;
use wutengine_shadercompiler::preprocessor::PreprocessErr;

mod compile;
mod types;

pub use types::*;

pub(crate) use compile::*;

pub use compile::register_precompiled;

pub use wutengine_shadercompiler::engine::CAMERA_PARAMS_BIND_GROUP_INDEX;
pub use wutengine_shadercompiler::engine::INSTANCE_PARAMS_BIND_GROUP_INDEX;
pub use wutengine_shadercompiler::engine::MATERIAL_PARAMS_BIND_GROUP_INDEX;

/// A shader source, used when configuring Materials. Its variants are compiled on demand.
#[derive(Debug, Clone)]
pub struct Shader {
    /// The source code, in the WutEngine shader format
    pub(crate) source: Arc<str>,

    /// The name and declared keywords
    pub(crate) info: ShaderInfo,
}

impl FromStr for Shader {
    type Err = PreprocessErr;

    /// Creates a shader from WutEngine shader source code
    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            info: source.parse()?,
            source: Arc::from(source),
        })
    }
}

impl Shader {
    /// The human-readable name of this shader
    #[inline]
    pub fn name(&self) -> &str {
        self.info.name()
    }
}

impl FromSerializedAsset for Shader {
    type Error = PreprocessErr;

    type Serialized = SerializedShader;

    fn from_serialized_asset(serialized: Self::Serialized) -> Result<Self, Self::Error> {
        serialized.source.parse()
    }
}
