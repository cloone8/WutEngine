//! Graphics pipeline caching

use alloc::sync::Arc;
use std::sync::LazyLock;
use wutengine_asset::assets::mesh::MeshTopology;

use smallvec::SmallVec;

use crate::graphics::shader::CompiledShaderId;

use super::GraphicsCache;

/// The global pipeline cache
static PIPELINE_CACHE: LazyLock<GraphicsCache<PipelineCacheKey, wgpu::RenderPipeline>> =
    LazyLock::new(Default::default);

/// The key identifying a [wgpu::RenderPipeline] in the pipeline cache
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PipelineCacheKey {
    /// The shader the pipeline uses
    pub(crate) shader: CompiledShaderId,

    /// The color targets the pipeline supports
    pub(crate) color_targets: SmallVec<[Option<wgpu::ColorTargetState>; 2]>,

    /// The topology of the mesh
    pub(crate) mesh_topology: MeshTopology,
}

/// Tries to find a given shader variant in the global cache
#[inline(always)]
pub(crate) fn find(key: &PipelineCacheKey) -> Option<Arc<wgpu::RenderPipeline>> {
    PIPELINE_CACHE.find(key)
}

/// Inserts the given compiled shader variant under the given key. If the variant already exists,
/// does not insert the new variant and simply returns the already existing one
#[inline(always)]
pub(crate) fn insert(
    key: PipelineCacheKey,
    pipeline: wgpu::RenderPipeline,
) -> Arc<wgpu::RenderPipeline> {
    PIPELINE_CACHE.insert(key, pipeline)
}
