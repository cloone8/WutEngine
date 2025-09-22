use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::GRAPHICS_MANAGER;

#[derive(Debug)]
pub(crate) struct PipelineCache {
    cache: RwLock<HashMap<PipelineCacheKey, Arc<wgpu::RenderPipeline>>>,
}

impl PipelineCache {
    pub(crate) fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::default()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineCacheKey {
    pub shader: String,
    pub shader_keyword_hash: u64,
    pub mesh_layout: crate::mesh::MeshVertexLayout,
}

pub fn get_cached_pipeline(key: &PipelineCacheKey) -> Option<Arc<wgpu::RenderPipeline>> {
    GRAPHICS_MANAGER
        .pipeline_cache
        .cache
        .read()
        .unwrap()
        .get(key)
        .cloned()
}

pub fn cache_pipeline(
    key: PipelineCacheKey,
    pipeline: wgpu::RenderPipeline,
) -> Arc<wgpu::RenderPipeline> {
    let mut cache = GRAPHICS_MANAGER.pipeline_cache.cache.write().unwrap();

    let as_arc = Arc::new(pipeline);

    cache.insert(key, as_arc.clone());

    as_arc
}
