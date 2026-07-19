//! Shader pipeline caching

use alloc::sync::Arc;
use std::sync::LazyLock;

use crate::shader::{CompiledShader, CompiledShaderId};

use super::GraphicsCache;

static SHADER_COMPILATION_CACHE: LazyLock<GraphicsCache<CompiledShaderId, CompiledShader>> =
    LazyLock::new(Default::default);

/// Tries to find a given shader variant in the global cache
#[inline]
pub(crate) fn find(key: &CompiledShaderId) -> Option<Arc<CompiledShader>> {
    SHADER_COMPILATION_CACHE.find(key)
}

/// Inserts the given compiled shader variant under the given key. If the variant already exists,
/// does not insert the new variant and simply returns the already existing one
#[inline]
pub(crate) fn insert(key: CompiledShaderId, variant: CompiledShader) -> Arc<CompiledShader> {
    SHADER_COMPILATION_CACHE.insert(key, variant)
}
