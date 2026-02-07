//! Builtin shaders

use std::sync::{Arc, LazyLock};

use crate::graphics::shaders::{Shader, ShaderId, ShaderParameter, ShaderParameterType};
use crate::map;

/// Fullscreen blit shader
pub static BLIT: LazyLock<Arc<Shader>> = LazyLock::new(|| {
    Arc::new(Shader {
        id: ShaderId::new(),
        name: "Blit".to_owned(),
        source: include_str!("shaders/blit.wgsl").to_owned(),
        allowed_keywords: map![],
        parameters: map![
            "source_sampler" => ShaderParameter { binding: (0, 0), ty: ShaderParameterType::Sampler, array_count: None },
            "source_texture" => ShaderParameter { binding: (0, 1), ty: ShaderParameterType::Texture2D, array_count: None }
        ],
    })
});
