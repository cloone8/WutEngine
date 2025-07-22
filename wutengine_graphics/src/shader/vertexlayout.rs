use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct ShaderVertexLayout {
    /// The location of the position vector
    pub position: Option<u32>,

    /// The location of the normal vector
    pub normal: Option<u32>,

    /// The location of the UV (texture coordinate) vector
    pub uv: Option<u32>,

    /// The location of the color vector
    pub color: Option<u32>,
}

impl ShaderVertexLayout {
    pub(crate) const fn num_attrs(&self) -> usize {
        self.position.is_some() as usize
            + self.normal.is_some() as usize
            + self.uv.is_some() as usize
            + self.color.is_some() as usize
    }
}
