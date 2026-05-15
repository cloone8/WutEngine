//! Sampler object caching

use alloc::sync::Arc;
use std::sync::LazyLock;

use crate::graphics::sampler::{Filtering, WrapModeType};

use super::GraphicsCache;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SamplerCacheKey {
    pub(crate) filtering: Filtering,
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
