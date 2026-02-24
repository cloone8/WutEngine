use core::num::NonZero;
use core::ops::RangeInclusive;
use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Shader {
    #[serde(skip)]
    id: usize,
    name: String,
    camera_params: bool,
    instance_params: bool,
    keywords: HashMap<String, RangeInclusive<u64>>,
    user_params: Vec<ShaderParameter>,
    source: ShaderSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "lowercase")]
enum ShaderParameter {
    Buffer {
        #[serde(rename = "type")]
        ty: ShaderBufferType,

        name: String,

        condition: ShaderParameterCondition,
    },
    Opaque {
        #[serde(rename = "type")]
        ty: ShaderOpaqueType,

        name: String,

        binding: NonZero<usize>,

        condition: ShaderParameterCondition,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ShaderBufferType {
    Vec4f,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ShaderOpaqueType {
    Sampler,
    #[serde(rename = "texture_2d")]
    Texture2D,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "lowercase")]
enum ShaderSource {
    Inline { content: String },
    File { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
struct ShaderParameterCondition(pub(crate) String);

fn main() {
    let desc: Shader = serde_json::from_str(include_str!("unlit.json")).unwrap();

    println!("{:#?}", desc);
}
