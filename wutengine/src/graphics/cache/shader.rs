//! Shader pipeline caching

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

use crate::graphics::shader::{CompiledShader, CompiledShaderId};

static SHADER_COMPILATION_CACHE: LazyLock<ShaderCompilationCache> = LazyLock::new(Default::default);

#[derive(Debug, Default)]
struct ShaderCompilationCache {
    cache: RwLock<HashMap<CompiledShaderId, Arc<CompiledShader>>>,
}

/// Tries to find a given shader variant in the global cache
#[inline]
pub(crate) fn find(key: &CompiledShaderId) -> Option<Arc<CompiledShader>> {
    let cache = SHADER_COMPILATION_CACHE.cache.read().unwrap();

    cache.get(key).map(Clone::clone)
}

/// Inserts the given compiled shader variant under the given key. If the variant already exists,
/// does not insert the new variant and simply returns the already existing one
#[inline]
pub(crate) fn insert(key: CompiledShaderId, variant: CompiledShader) -> Arc<CompiledShader> {
    let mut cache = SHADER_COMPILATION_CACHE.cache.write().unwrap();

    if let Some(existing) = cache.get(&key) {
        existing.clone()
    } else {
        let as_arc = Arc::new(variant);
        cache.insert(key, as_arc.clone());

        as_arc
    }
}
