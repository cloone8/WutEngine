//! Sampler object caching

use std::sync::{Arc, LazyLock};

use crate::graphics::sampler::Sampler;

use super::GraphicsCache;

static SAMPLER_CACHE: LazyLock<GraphicsCache<Sampler, wgpu::Sampler>> =
    LazyLock::new(Default::default);

/// Tries to find a given sampler object in the global cache
#[inline(always)]
pub(crate) fn find(sampler: &Sampler) -> Option<Arc<wgpu::Sampler>> {
    SAMPLER_CACHE.find(sampler)
}

/// Inserts a new sampler object into the global cache under the given key.
/// If a sampler object is already present in the global cache, does not replace it and simply
/// returns the already present object.
#[inline(always)]
pub(crate) fn insert(sampler: Sampler, sampler_object: wgpu::Sampler) -> Arc<wgpu::Sampler> {
    SAMPLER_CACHE.insert(sampler, sampler_object)
}
