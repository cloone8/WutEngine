use crate::window::Window;

/// The target surface on which a [`Camera`] will render its viewport
#[derive(Debug, Clone, Copy)]
pub enum CameraTarget {
    /// This camera renders to the given [`Window`]
    Window(Window),
}

impl CameraTarget {
    /// Returns the size (in pixels) of this target
    pub fn size(&self) -> (u32, u32) {
        match self {
            Self::Window(window) => window.get_size(),
        }
    }
}
