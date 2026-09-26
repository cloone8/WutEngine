//! Builtin shaders

use std::sync::Arc;
use std::sync::LazyLock;

use crate::graphics::shader::Shader;

/// Fullscreen blit shader
pub static BLIT: LazyLock<Arc<Shader>> = LazyLock::new(|| {
    Arc::new(
        include_str!("blit.wgsl")
            .parse::<Shader>()
            .expect("Invalid built-in blit shader"),
    )
});

/// Unlit shader
pub static UNLIT: LazyLock<Arc<Shader>> = LazyLock::new(|| {
    Arc::new(
        include_str!("unlit.wgsl")
            .parse::<Shader>()
            .expect("Invalid built-in unlit shader"),
    )
});
