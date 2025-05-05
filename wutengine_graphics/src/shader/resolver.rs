use super::{Shader, ShaderVariantId};

/// A shader resolver. Responsible for abstracting away the source of a set of shaders, allowing them
/// to be sourced from different places (disk, embedded, different binary formats, etc.)
pub trait ShaderResolver: 'static + Send + Sync {
    /// Find the shader set corresponding to the given ID.
    /// If the shader with the given ID is not found, the resolver is allowed to
    /// use [ShaderId::without_keywords] to find the raw form of the shader instead,
    /// and return that in its uncompiled form. The caller is then responsible for
    /// compiling the shader, if required.
    fn find_set(&self, id: &ShaderVariantId) -> Option<&Shader>;
}
