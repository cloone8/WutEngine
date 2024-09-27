use std::collections::HashMap;

use glam::Mat4;

use crate::color::Color;
use crate::shader::ShaderSetId;

#[derive(Debug, Clone)]
pub struct MaterialData {
    pub shader: ShaderSetId,
    pub parameters: HashMap<String, MaterialParameter>,
}

#[derive(Debug, Clone)]
pub enum MaterialParameter {
    Color(Color),
    Mat4(Mat4),
}
