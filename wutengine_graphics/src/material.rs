use std::collections::HashMap;

use glam::Mat4;

use crate::color::Color;
use crate::renderer::RendererTextureId;
use crate::shader::ShaderId;

#[derive(Debug, Clone, Default)]
pub struct MaterialData {
    pub shader: Option<ShaderId>,
    pub parameters: HashMap<String, MaterialParameter>,
}

#[derive(Debug, Clone)]
pub enum MaterialParameter {
    Color(Color),
    Mat4(Mat4),
    Texture(RendererTextureId),
}
