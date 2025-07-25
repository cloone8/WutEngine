use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use wutengine_event::EventSubscription;

use crate::{DeviceLostEvent, GRAPHICS_MANAGER};

#[derive(Debug)]
pub(crate) struct PipelineCache {
    device_lost_subscription: EventSubscription,
    cache: RwLock<HashMap<PipelineCacheKey, Arc<wgpu::RenderPipeline>>>,
}

impl PipelineCache {
    pub(crate) fn new() -> Self {
        Self {
            device_lost_subscription: wutengine_event::subscribe::<DeviceLostEvent>(on_device_lost),
            cache: RwLock::new(HashMap::default()),
        }
    }
}

fn on_device_lost(_event: &DeviceLostEvent) {
    log::warn!("Clearing GPU pipeline cache because the GPU device was lost");

    GRAPHICS_MANAGER
        .pipeline_cache
        .cache
        .write()
        .unwrap()
        .clear();
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
