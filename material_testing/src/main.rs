use core::num::NonZero;
use core::ops::RangeInclusive;
use std::collections::HashMap;
use std::path::PathBuf;

use material_shadercomp::CompInput;
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

impl Shader {
    pub fn load_source(&mut self) {
        if let ShaderSource::File { path } = &self.source {
            let content = std::fs::read_to_string(path).unwrap();
            self.source = ShaderSource::Inline { content };
        }
    }

    pub fn get_source(&self) -> &str {
        if let ShaderSource::Inline { content } = &self.source {
            content.as_str()
        } else {
            panic!("Invalid source");
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "lowercase")]
enum ShaderParameter {
    Buffer {
        #[serde(rename = "type")]
        ty: ShaderBufferType,

        name: String,

        condition: Option<ShaderParameterCondition>,
    },
    Opaque {
        #[serde(rename = "type")]
        ty: ShaderOpaqueType,

        name: String,

        binding: NonZero<usize>,

        condition: Option<ShaderParameterCondition>,
    },
}

impl ShaderParameter {
    pub fn get_condition(&self) -> Option<&ShaderParameterCondition> {
        match self {
            Self::Buffer { condition, .. } => condition.as_ref(),
            Self::Opaque { condition, .. } => condition.as_ref(),
        }
    }
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
    let mut desc: Shader = serde_json::from_str(include_str!("unlit.json")).unwrap();
    println!("{:#?}", desc);

    desc.load_source();

    let mut keywords = HashMap::default();

    keywords.insert("HAS_COLOR_MAP".to_owned(), 1);

    let user_params: Vec<Option<&str>> = Vec::from_iter(
        desc.user_params
            .iter()
            .map(|p| p.get_condition().map(|c| c.0.as_str())),
    );

    let output = material_shadercomp::compile::<_, FakeHasher>(CompInput {
        id: desc.id,
        source: desc.get_source(),
        keywords: &keywords,
        user_params: &user_params,
        per_camera_block: "",
        per_instance_block: "",
    });

    println!("{:#?}", output);
}

struct FakeHasher;

impl<T> material_shadercomp::ShaderHasher<T> for FakeHasher {
    fn hash_source_id(_id: T) -> u64 {
        420
    }

    fn hash_keywords<S: AsRef<str>>(_keywords: &HashMap<S, u64>) -> u64 {
        69
    }
}
