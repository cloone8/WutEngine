//! Builtin shaders

use std::sync::{Arc, LazyLock};

use wutengine_assets::{
    FromSerializedAsset,
    assets::shader::{SerializedShader, ShaderSource},
};

use crate::graphics::shader::Shader;

/// Macro to automatically create a [Shader] from a descriptor and source file,
/// overriding the "source" field of the shader descriptor to be inline
macro_rules! from_descriptor_and_source {
    ($name:literal) => {{
        let descriptor = include_str!(concat!($name, ".json"));
        let source = include_str!(concat!($name, ".wgsl"));

        let mut shader = serde_json::from_str::<SerializedShader>(descriptor).expect(concat!(
            "Invalid built-in shader: \"",
            $name,
            "\""
        ));

        shader.source = ShaderSource::Inline {
            content: source.to_owned(),
        };

        Arc::new(Shader::from_serialized_asset(shader).unwrap())
    }};
}

/// Fullscreen blit shader
pub static BLIT: LazyLock<Arc<Shader>> = LazyLock::new(|| from_descriptor_and_source!("blit"));

/// Unlit shader
pub static UNLIT: LazyLock<Arc<Shader>> = LazyLock::new(|| from_descriptor_and_source!("unlit"));
