//! Material related functionality

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use glam::{Mat4, Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};

use crate::asset::{Asset, AssetHandle, SerializedAsset};

use super::sampler::Sampler;
use super::shader::{CompiledShader, Shader};
use super::texture::Texture;
use super::{BindGroup, shader};

/// A WutEngine material. Contains a specific shader variant and a set of parameters for drawing.
#[derive(Debug)]
pub struct Material {
    /// The shader
    pub(crate) shader: Arc<Shader>,

    /// The active keywords for the shader this material uses
    pub(crate) keywords: HashMap<String, u64>,

    /// The compiled shader
    pub(crate) compiled_shader: Arc<CompiledShader>,

    /// The bind group for the user parameters of this material
    pub(crate) user_bind_group: BindGroup,
}

impl Material {
    /// Creates a new material that uses the given shader with the given keyword values set
    pub(crate) fn new(shader: Arc<Shader>, keywords: HashMap<String, u64>) -> Self {
        let compiled_shader =
            shader::compile(&shader, &keywords).expect("Failed to compile shader");

        Self {
            shader,
            keywords,
            user_bind_group: BindGroup::new(
                "Material User Bind Group".to_string(),
                compiled_shader.user_bind_group_layout.clone(),
                &compiled_shader.parameters,
            ),
            compiled_shader,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedMaterial {
    pub shader: AssetHandle<Shader>,
    pub keywords: HashMap<String, u64>,
    pub parameters: HashMap<String, MaterialParameter>,
}

impl SerializedAsset for SerializedMaterial {
    type AssetType = Material;
}

impl Asset for Material {
    type Serialized = SerializedMaterial;

    type FromSerializedErr = Infallible;

    fn from_serialized(serialized: &Self::Serialized) -> Result<Self, Self::FromSerializedErr>
    where
        Self: Sized,
    {
        let mut mat = Material::new(
            serialized
                .shader
                .get_arc()
                .expect("Shader asset not yet loaded"),
            serialized.keywords.clone(),
        );

        let queue = super::queue();

        for (param_name, param_value) in &serialized.parameters {
            if let Err(e) =
                mat.user_bind_group
                    .set_parameter(param_name.as_str(), param_value.clone(), queue)
            {
                log::warn!(
                    "Error setting material parameter {param_name} during deserialization: {e}"
                );
            }
        }

        Ok(mat)
    }
}

/// A material parameter value
#[derive(
    Debug,
    Clone,
    PartialEq,
    derive_more::IsVariant,
    derive_more::Unwrap,
    derive_more::TryUnwrap,
    derive_more::From,
    Serialize,
    Deserialize,
)]
pub enum MaterialParameter {
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

    /// 4x4 matrix
    Mat4(Mat4),

    /// 2D texture
    Texture2D(AssetHandle<Texture>),

    /// Sampler
    Sampler(AssetHandle<Sampler>),
}
