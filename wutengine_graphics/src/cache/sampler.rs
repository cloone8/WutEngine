//! Sampler object caching

use alloc::sync::Arc;
use std::sync::LazyLock;
use wutengine_asset::assets::sampler::FilterMode;
use wutengine_asset::assets::sampler::WrapModeType;

use super::GraphicsCache;

/// Cache key for a [crate::sampler::Sampler]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SamplerCacheKey {
    /// Filter mode
    pub(crate) filtering: FilterMode,

    /// Wrapping mode
    pub(crate) wrapping: WrapModeType,
}

static SAMPLER_CACHE: LazyLock<GraphicsCache<SamplerCacheKey, wgpu::Sampler>> =
    LazyLock::new(Default::default);

/// Tries to find a given sampler object in the global cache
#[inline(always)]
pub(crate) fn find(sampler: &SamplerCacheKey) -> Option<Arc<wgpu::Sampler>> {
    SAMPLER_CACHE.find(sampler)
}

/// Inserts a new sampler object into the global cache under the given key.
/// If a sampler object is already present in the global cache, does not replace it and simply
/// returns the already present object.
#[inline(always)]
pub(crate) fn insert(
    sampler: SamplerCacheKey,
    sampler_object: wgpu::Sampler,
) -> Arc<wgpu::Sampler> {
    SAMPLER_CACHE.insert(sampler, sampler_object)
}
