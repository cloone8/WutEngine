/// A descriptor for the layout of a mesh vertex buffer
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) struct MeshVertexLayout {
    /// The total stride between consecutive vertices
    pub stride: u64,

    /// The offset of the position attribute within a vertex (in bytes)
    pub position: Option<u64>,

    /// The location of the normal attribute within a vertex (in bytes)
    pub normal: Option<u64>,

    /// The location of the UV (texture coordinate) attribute within a vertex (in bytes)
    pub uv: Option<u64>,

    /// The offset of the color attribute within a vertex (in bytes)
    pub color: Option<u64>,
}

impl MeshVertexLayout {
    pub(crate) const EMPTY: Self = Self {
        stride: 0,
        position: None,
        color: None,
        normal: None,
        uv: None,
    };
}
