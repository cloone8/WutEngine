use std::collections::HashMap;

use glam::{Vec2, Vec3, Vec4};

use crate::color::Color;
use crate::shader::ShaderSetId;

#[derive(Debug)]
pub struct MaterialData {
    pub shader: ShaderSetId,
    pub parameters: HashMap<String, MaterialParameter>,
}

#[derive(Debug, Clone)]
pub enum MaterialParameter {
    Array(Vec<MaterialParameter>),
    Float(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Color(Color),
}
