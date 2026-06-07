//! Material related functionality

use alloc::sync::Arc;
use core::convert::Infallible;
use std::collections::HashMap;
use wutengine_asset::assets::material::SerializedMaterial;
use wutengine_math::Color;
use wutengine_util_macro::unique_id_type32;

use serde::{Deserialize, Serialize};
use wutengine_math::{Mat4, Vec2, Vec3, Vec4};
use wutengine_util_macro::VariantName;

use wutengine_asset::{Asset, AssetHandle};

use super::sampler::Sampler;
use super::shader::{CompiledShader, Shader};
use super::texture::Texture;
use super::{BindGroup, shader};

unique_id_type32! {
    /// Globally unique identifier for a [Material]
    pub MaterialId
}

/// A WutEngine material. Contains a specific shader variant and a set of parameters for drawing.
#[derive(Debug)]
pub struct Material {
    /// The unique ID of this material
    pub(crate) id: MaterialId,

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
    pub fn new(shader: Arc<Shader>, keywords: HashMap<String, u64>) -> Self {
        let compiled_shader =
            shader::compile(&shader, &keywords).expect("Failed to compile shader");

        Self {
            id: MaterialId::new(),
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

    /// The unique ID for this material
    #[inline(always)]
    pub fn id(&self) -> MaterialId {
        self.id
    }

    /// The raw bind group
    #[inline(always)]
    pub fn raw_bind_group(&self) -> &BindGroup {
        &self.user_bind_group
    }

    /// The mutable raw bind group
    #[inline(always)]
    pub fn raw_bind_group_mut(&mut self) -> &mut BindGroup {
        &mut self.user_bind_group
    }

    /// The compiled shader
    #[inline(always)]
    pub fn compiled_shader(&self) -> &CompiledShader {
        &self.compiled_shader
    }
}

impl Clone for Material {
    fn clone(&self) -> Self {
        Self {
            id: MaterialId::new(),
            shader: self.shader.clone(),
            keywords: self.keywords.clone(),
            compiled_shader: self.compiled_shader.clone(),
            user_bind_group: self.user_bind_group.clone(),
        }
    }
}

impl Asset for Material {
    type Serialized = SerializedMaterial<Shader, MaterialParameter>;

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
                log::error!(
                    "Error setting material parameter {param_name} during deserialization: {e}"
                );
            }
        }

        mat.user_bind_group.update_bind_group(super::device());

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
    VariantName,
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

    /// A 4 component color value
    Color(Color),

    /// 4x4 matrix
    Mat4(Mat4),

    /// 2D texture
    Texture2D(AssetHandle<Texture>),

    /// Sampler
    Sampler(AssetHandle<Sampler>),
}
