use super::{Shader, ShaderId};

/// A shader resolver. Responsible for abstracting away the source of a set of shaders, allowing them
/// to be sourced from different places (disk, embedded, different binary formats, etc.)
pub trait ShaderResolver: 'static + Send + Sync {
    /// Find the shader set corresponding to the given ID
    fn find_set(&self, id: &ShaderId) -> Option<&Shader>;
}
