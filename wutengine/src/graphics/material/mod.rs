use core::num::NonZero;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::graphics::cache;
use crate::graphics::shader::compile;

use super::BindGroup;
use super::shader::{CompiledShader, Shader};

#[derive(Debug)]
pub(crate) struct Material {
    pub(crate) shader: Shader,
    pub(crate) keywords: HashMap<String, u64>,
    pub(crate) compiled_shader: Arc<CompiledShader>,
    pub(crate) user_bind_group: BindGroup,
}

impl Material {
    pub(crate) fn new(shader: Shader, keywords: HashMap<String, u64>) -> Self {
        let compiled_shader = compile(&shader, &keywords).expect("Failed to compile shader");

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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    derive_more::IsVariant,
    derive_more::Unwrap,
    derive_more::TryUnwrap,
    derive_more::From,
)]
pub enum MaterialParameter {
    Uint(u32),
    Int(i32),
    Flt(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Mat4(Mat4),
}
