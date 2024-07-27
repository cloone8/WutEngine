use super::Shader;

pub const UNLIT: Shader = Shader {
    source: super::ShaderSource::Builtin {
        identifier: "unlit",
    },
    available_keywords: Vec::new(),
};
