//! Module for the data of a WutEngine material. A material describes the way a given mesh is rendered

use std::collections::HashMap;

use glam::Mat4;

use crate::color::Color;
use crate::renderer::RendererTextureId;
use crate::shader::ShaderId;

/// The data of a material
#[derive(Debug, Clone, Default)]
pub struct MaterialData {
    /// The shader used to render this material. If [None], this material will not render
    pub shader: Option<ShaderId>,

    /// Any material parameters
    pub parameters: HashMap<String, MaterialParameter>,
}

/// The value of a material parameter in a [MaterialData]
#[derive(Debug, Clone)]
pub enum MaterialParameter {
    /// A boolean
    Boolean(bool),

    /// A color
    Color(Color),

    /// A matrix
    Mat4(Mat4),

    /// A texture
    Texture(RendererTextureId),
}
