//! Shader pipeline caching

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

use crate::graphics::shaders::CompiledShader;

static SHADER_COMPILATION_CACHE: LazyLock<ShaderCompilationCache> = LazyLock::new(Default::default);

#[derive(Debug, Default)]
struct ShaderCompilationCache {
    cache: RwLock<HashMap<ShaderCompilationCacheKey, Arc<CompiledShader>>>,
}

/// Cache key for a [CompiledShader]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ShaderCompilationCacheKey {
    /// The hash for the main shader ID
    pub(crate) shader_id_hash: u64,

    /// The hash for the enabled keywords for the needed variant
    pub(crate) keyword_hash: u64,
}

/// Tries to find a given shader variant in the global cache
#[inline]
pub(crate) fn find(key: &ShaderCompilationCacheKey) -> Option<Arc<CompiledShader>> {
    let cache = SHADER_COMPILATION_CACHE.cache.read().unwrap();

    cache.get(key).map(Clone::clone)
}

/// Inserts the given compiled shader variant under the given key. If the variant already exists,
/// does not insert the new variant and simply returns the already existing one
#[inline]
pub(crate) fn insert(
    key: ShaderCompilationCacheKey,
    variant: CompiledShader,
) -> Arc<CompiledShader> {
    let mut cache = SHADER_COMPILATION_CACHE.cache.write().unwrap();

    if let Some(existing) = cache.get(&key) {
        existing.clone()
    } else {
        let as_arc = Arc::new(variant);
        cache.insert(key, as_arc.clone());

        as_arc
    }
}
