//! Builtin shaders

use std::sync::LazyLock;

use crate::graphics::shaders::Shader;
use crate::map;

/// Fullscreen blit shader
pub static BLIT: LazyLock<Shader> = LazyLock::new(|| Shader {
    id: "blit".to_owned(),
    source: include_str!("shaders/blit.wgsl").to_owned(),
    allowed_keywords: map![],
});
