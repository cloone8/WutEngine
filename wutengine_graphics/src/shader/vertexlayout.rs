use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct ShaderVertexLayout {
    /// The location of the position vector, if used
    pub position: Option<u32>,

    /// The location of the normal vector, if used
    pub normal: Option<u32>,

    /// The location of the UV (texture coordinate) vector, if used
    pub uv: Option<u32>,

    /// The location of the color vector, if used
    pub color: Option<u32>,
}

impl ShaderVertexLayout {
    pub const fn num_attrs(&self) -> usize {
        self.position.is_some() as usize
            + self.normal.is_some() as usize
            + self.uv.is_some() as usize
            + self.color.is_some() as usize
    }
}
