use wutengine_graphics::rendertexture::RenderTexture;

use crate::window::Window;

/// The target surface on which a [`Camera`](super::Camera) will render its viewport
#[derive(Debug, Clone)]
pub enum CameraTarget {
    /// This camera renders to the given [`Window`]
    Window(Window),

    /// This camera renders to the given [`RenderTexture`]
    Texture(RenderTexture),
}

impl CameraTarget {
    /// Returns the size (in pixels) of this target
    pub fn size(&self) -> (u32, u32) {
        match self {
            Self::Window(window) => window.get_size(),
            Self::Texture(render_texture) => render_texture.size(),
        }
    }
}
