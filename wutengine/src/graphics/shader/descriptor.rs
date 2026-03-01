use core::ops::RangeInclusive;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderDescriptor {
    pub name: String,
    pub camera_params: bool,
    pub instance_params: bool,
    pub keywords: HashMap<String, RangeInclusive<u64>>,
    pub user_params: Vec<ShaderParameter>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ShaderParameter {
    Opaque {
        #[serde(rename = "type")]
        ty: ShaderOpaqueType,

        name: String,

        #[serde(default)]
        visibility: ShaderParamVisibility,

        condition: ShaderParameterCondition,

        binding: u32,
    },
    Buffer {
        #[serde(rename = "type")]
        ty: ShaderBufferType,

        name: String,

        #[serde(default)]
        visibility: ShaderParamVisibility,

        condition: ShaderParameterCondition,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShaderOpaqueType {
    Sampler,
    #[serde(rename = "texture_2d")]
    Texture2D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShaderBufferType {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShaderParamVisibility {
    Vertex,
    Fragment,

    #[default]
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct ShaderParameterCondition(pub(crate) String);
