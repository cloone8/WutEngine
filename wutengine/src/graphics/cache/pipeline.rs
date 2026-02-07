//! Graphics pipeline caching

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

/// The global pipeline cache
static PIPELINE_CACHE: LazyLock<PipelineCache> = LazyLock::new(Default::default);

#[derive(Debug, Default)]
struct PipelineCache {
    cache: RwLock<HashMap<PipelineCacheKey, Arc<wgpu::RenderPipeline>>>,
}

/// The key identifying a [wgpu::RenderPipeline] in the pipeline cache
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PipelineCacheKey {
    /// The shader the pipeline uses
    pub(crate) shader: super::shader::ShaderCompilationCacheKey,
}

/// Tries to find a given shader variant in the global cache
#[inline]
pub(crate) fn find(key: &PipelineCacheKey) -> Option<Arc<wgpu::RenderPipeline>> {
    let cache = PIPELINE_CACHE.cache.read().unwrap();

    cache.get(key).map(Clone::clone)
}

/// Inserts the given compiled shader variant under the given key. If the variant already exists,
/// does not insert the new variant and simply returns the already existing one
#[inline]
pub(crate) fn insert(
    key: PipelineCacheKey,
    pipeline: wgpu::RenderPipeline,
) -> Arc<wgpu::RenderPipeline> {
    let mut cache = PIPELINE_CACHE.cache.write().unwrap();

    if let Some(existing) = cache.get(&key) {
        existing.clone()
    } else {
        let as_arc = Arc::new(pipeline);
        cache.insert(key, as_arc.clone());

        as_arc
    }
}
