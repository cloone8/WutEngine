use std::path::PathBuf;

use include_dir::File;
use wutengine_graphics::shader::resolver::ShaderResolver;
use wutengine_graphics::shader::{ShaderSet, ShaderSetId};

use crate::embedded;

pub struct EmbeddedShaderResolver;

impl ShaderResolver for EmbeddedShaderResolver {
    fn find_set(&self, id: &ShaderSetId) -> Option<ShaderSet> {
        let dir_path = PathBuf::from(id.to_string());
        let set_dir = embedded::SHADERS.get_dir(&dir_path)?;

        Some(ShaderSet {
            id: id.clone(),
            vertex_source: set_dir
                .get_file(dir_path.join("vertex.glsl"))
                .map(string_from_include_dir_file),
            fragment_source: set_dir
                .get_file(dir_path.join("fragment.glsl"))
                .map(string_from_include_dir_file),
        })
    }
}

fn string_from_include_dir_file(file: &File) -> String {
    file.contents_utf8()
        .expect("Non UTF-8 shader source file")
        .to_owned()
}
