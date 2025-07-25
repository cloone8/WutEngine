use std::sync::LazyLock;

use wutengine_graphics::shader::{ShaderSource, ShaderVertexLayout};
use wutengine_util::map;

/// The default unlit shader
pub static UNLIT: LazyLock<ShaderSource> = LazyLock::new(|| ShaderSource {
    name: "unlit".to_owned(),
    source: include_str!("unlit.wgsl").to_owned(),
    available_keywords: map! {
        "HAS_COLOR_MAP" => 0..=1
    },
    vertex_layout: ShaderVertexLayout {
        position: Some(0),
        normal: None,
        uv: None,
        color: Some(1),
    },
});
