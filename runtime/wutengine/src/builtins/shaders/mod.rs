//! Builtin shaders

#[cfg(feature = "std")]
use std::sync::LazyLock;

use alloc::string::ToString;
use wutengine_assets::{
    FromSerializedAsset,
    assets::shader::{
        SerializedShader, ShaderBufferParameterType, ShaderDefaultParameters, ShaderKeyword,
        ShaderOpaqueParameterType, ShaderParameter, ShaderParameterCondition, ShaderSource,
        ShaderVertexAttribute, ShaderVertexAttributeType,
    },
};

use alloc::sync::Arc;
use alloc::vec;

use crate::graphics::shader::Shader;

/// Fullscreen blit shader
pub static BLIT: LazyLock<Arc<Shader>> = LazyLock::new(|| {
    let source = include_str!("blit.wgsl");

    let shader = SerializedShader {
        name: "Blit".to_string(),
        vertex_attributes: vec![],
        default_parameters: ShaderDefaultParameters {
            camera: false,
            instance: false,
        },
        keywords: Default::default(),
        parameters: vec![
            ShaderParameter::Opaque {
                ty: ShaderOpaqueParameterType::Sampler,
                name: "source_sampler".to_string(),
                condition: None,
            },
            ShaderParameter::Opaque {
                ty: ShaderOpaqueParameterType::Texture2D,
                name: "source_texture".to_string(),
                condition: None,
            },
        ],
        source: ShaderSource::Inline {
            content: source.to_string(),
        },
    };

    Arc::new(Shader::from_serialized_asset(shader).unwrap())
});

/// Unlit shader
pub static UNLIT: LazyLock<Arc<Shader>> = LazyLock::new(|| {
    let source = include_str!("unlit.wgsl");

    let shader = SerializedShader {
        name: "Unlit".to_string(),
        vertex_attributes: vec![
            ShaderVertexAttribute {
                ty: ShaderVertexAttributeType::Position,
                location: 0,
                condition: None,
            },
            ShaderVertexAttribute {
                ty: ShaderVertexAttributeType::Uv { channel: 0 },
                location: 1,
                condition: Some(ShaderParameterCondition("HAS_COLOR_MAP != 0".to_string())),
            },
        ],
        default_parameters: ShaderDefaultParameters {
            camera: true,
            instance: true,
        },
        keywords: wutengine_util::map! {
            "HAS_COLOR_MAP"=> ShaderKeyword {
                default: 0,
                allowed: 0..=1,
            }
        },
        parameters: vec![
            ShaderParameter::Buffer {
                ty: ShaderBufferParameterType::Vec4f,
                name: "base_color".to_string(),
                condition: None,
            },
            ShaderParameter::Opaque {
                ty: ShaderOpaqueParameterType::Sampler,
                name: "color_map_sampler".to_string(),
                condition: Some(ShaderParameterCondition("HAS_COLOR_MAP != 0".to_string())),
            },
            ShaderParameter::Opaque {
                ty: ShaderOpaqueParameterType::Texture2D,
                name: "color_map_texture".to_string(),
                condition: Some(ShaderParameterCondition("HAS_COLOR_MAP != 0".to_string())),
            },
        ],
        source: ShaderSource::Inline {
            content: source.to_string(),
        },
    };
    Arc::new(Shader::from_serialized_asset(shader).unwrap())
});
