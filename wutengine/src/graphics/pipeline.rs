//! Graphics pipeline functions

use alloc::sync::Arc;

use wutengine_util_macro::unique_id_type64;

use crate::graphics::{self, GFX_DEVICE};

use super::cache;
use super::cache::pipeline::PipelineCacheKey;
use super::material::Material;
use super::mesh::MeshTopology;

unique_id_type64! {
    /// Unique ID for a render pipeline. Mostly used for debug labels
    PipelineId
}

/// An error while trying to retrieve a render pipeline
#[derive(Debug, derive_more::Display, derive_more::From, derive_more::Error)]
pub enum GetPipelineErr {
    /// Error during shader compilation
    #[display("Error while compiling shader for pipeline: {}", _0)]
    ShaderCompile(Box<graphics::shader::CompileErr>),
}

/// Given the set of input parameters, returns a matching [wgpu::RenderPipeline].
/// A cached copy of the pipeline is returned if possible. If not, creates a new pipeline.
///
/// If a new pipeline is created, an attempt is made to get the cached copy of the compiled shader. If this
/// cached copy does not exist, the shader is compiled and cached.
pub fn get_pipeline(
    material: &Material,
    topology: MeshTopology,
    color_targets: &[Option<wgpu::ColorTargetState>],
) -> Result<Arc<wgpu::RenderPipeline>, GetPipelineErr> {
    profiling::function_scope!();

    let pipeline_cache_key = PipelineCacheKey {
        shader: material.compiled_shader.id,
        color_targets: color_targets.into(),
        mesh_topology: topology,
    };

    if let Some(cached_pipeline) = cache::pipeline::find(&pipeline_cache_key) {
        return Ok(cached_pipeline);
    }

    let pipeline_id = PipelineId::new();
    let compiled_shader = material.compiled_shader.as_ref();

    log::debug!(
        "Creating new pipeline with ID {pipeline_id} for shader variant {}",
        compiled_shader
    );

    let pipeline_layout = &compiled_shader.pipeline_layout;

    let native_shader_module = &compiled_shader.module;

    let pipeline = GFX_DEVICE.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(
            format!(
                "Shader variant {} pipeline {}",
                compiled_shader, pipeline_id
            )
            .as_str(),
        ),
        layout: Some(pipeline_layout),

        vertex: wgpu::VertexState {
            module: native_shader_module,
            entry_point: None,
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: native_shader_module,
            entry_point: None,
            compilation_options: Default::default(),
            targets: color_targets,
        }),
        primitive: wgpu::PrimitiveState {
            topology: topology.to_wgpu(),
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: None,
    });

    Ok(cache::pipeline::insert(pipeline_cache_key, pipeline))
}
