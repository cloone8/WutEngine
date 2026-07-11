//! Builtin shaders

use std::sync::LazyLock;

use crate::graphics::shader::Shader;
use wutengine_asset::Asset;
use wutengine_asset::AssetHandle;
use wutengine_asset::assets::shader::SerializedShader;
use wutengine_asset::assets::shader::ShaderSource;

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

        AssetHandle::from(Shader::from_serialized(&shader).unwrap())
    }};
}

/// Fullscreen blit shader
pub static BLIT: LazyLock<AssetHandle<Shader>> =
    LazyLock::new(|| from_descriptor_and_source!("blit"));

/// Unlit shader
pub static UNLIT: LazyLock<AssetHandle<Shader>> =
    LazyLock::new(|| from_descriptor_and_source!("unlit"));
