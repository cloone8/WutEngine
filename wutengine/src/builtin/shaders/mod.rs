use std::sync::LazyLock;

use wutengine_graphics::shader::{PossibleKeywordValue, ShaderSource};
use wutengine_util::map;

/// The default unlit shader
pub static UNLIT: LazyLock<ShaderSource> = LazyLock::new(|| ShaderSource {
    name: "unlit".to_owned(),
    code: include_str!("unlit.wgsl").to_owned(),
    available_keywords: map! {
        "HAS_COLOR_MAP" => PossibleKeywordValue::Bool
    },
});
