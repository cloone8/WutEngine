//! Material asset

use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use wutengine_math::Color;
use wutengine_math::Mat4;
use wutengine_math::Vec2;
use wutengine_math::Vec3;
use wutengine_math::Vec4;

use crate::AssetRef;
use crate::SerializedAsset;

use super::sampler::SerializedSampler;
use super::shader::SerializedShader;
use super::texture::SerializedTexture;

/// The data for a single material
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedMaterial {
    /// The shader used by this material
    pub shader: AssetRef<SerializedShader>,

    /// The set keyword values for this material
    pub keywords: HashMap<String, u64>,

    /// The parameter values for this material
    pub parameters: HashMap<String, SerializedMaterialParameter>,
}

impl SerializedAsset for SerializedMaterial {
    const ID: uuid::NonNilUuid =
        uuid::NonNilUuid::new(uuid::uuid!("1b49f3cc-7779-431b-9bc3-ad98d275abce")).unwrap();
}

/// A material parameter value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SerializedMaterialParameter {
    /// Unsigned 32-bit integer
    Uint(u32),

    /// Signed 32-bit integer
    Int(i32),

    /// 32-bit float
    Flt(f32),

    /// Two-component float vector
    Vec2(Vec2),

    /// Three-component float vector
    Vec3(Vec3),

    /// Four-component float vector
    Vec4(Vec4),

    /// A 4 component color value
    Color(Color),

    /// 4x4 matrix
    Mat4(Mat4),

    /// 2D texture
    Texture2D(AssetRef<SerializedTexture>),

    /// Sampler
    Sampler(AssetRef<SerializedSampler>),
}
