//! Graphics pipeline functions

use alloc::sync::Arc;

use smallvec::SmallVec;
use wutengine_assets::assets::mesh::MeshTopology;
use wutengine_util_macro::unique_id_type64;

use super::{cache, cache::pipeline::PipelineCacheKey, material::Material};
use crate::{GFX_DEVICE, PIPELINE_CACHE, label, mesh::asset_topology_to_wgpu, shader};

unique_id_type64! {
    /// Unique ID for a render pipeline. Mostly used for debug labels
    PipelineId
}

/// An error while trying to retrieve a render pipeline
#[derive(Debug, derive_more::Display, derive_more::From, derive_more::Error)]
pub enum GetPipelineErr {
    /// Error during shader compilation
    #[display("Error while compiling shader for pipeline: {}", _0)]
    ShaderCompile(alloc::boxed::Box<shader::CompileErr>),
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
    let pipeline_cache_key = PipelineCacheKey {
        shader: material.compiled_shader.id,
        color_targets: color_targets.into(),
        mesh_topology: topology,
    };

    if let Some(cached_pipeline) = cache::pipeline::find(&pipeline_cache_key) {
        return Ok(cached_pipeline);
    }

    profiling::scope!("Create new pipeline");

    let pipeline_id = PipelineId::new();
    let compiled_shader = material.compiled_shader.as_ref();

    log::debug!(
        "Creating new pipeline with ID {pipeline_id} for shader variant {}",
        compiled_shader
    );

    let pipeline_layout = &compiled_shader.pipeline_layout;

    let native_shader_module = &compiled_shader.module;

    // Create the vertex state buffer layout
    const STACK_ATTRS: usize = 8;

    let mut vertex_buffer_attributes = SmallVec::<[_; STACK_ATTRS]>::new_const();
    vertex_buffer_attributes.reserve_exact(compiled_shader.vertex_attributes.len());

    for attr_info in compiled_shader.vertex_attributes.values() {
        vertex_buffer_attributes.push(*attr_info);
    }

    let mut vertex_state_buffers = SmallVec::<[_; STACK_ATTRS]>::new_const();
    vertex_state_buffers.reserve_exact(compiled_shader.vertex_attributes.len());

    for (i, attr_info) in compiled_shader.vertex_attributes.values().enumerate() {
        assert!(
            attr_info
                .format
                .size()
                .is_multiple_of(wgpu::VERTEX_ALIGNMENT),
            "Vertex data types with alignments smaller than wgpu::VERTEX_ALIGNMENT are not yet supported"
        );

        vertex_state_buffers.push(Some(wgpu::VertexBufferLayout {
            array_stride: attr_info.format.size(),
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &vertex_buffer_attributes[i..i + 1],
        }));
    }

    // Combine all info into a pipeline descriptor, and create it
    let pipeline = GFX_DEVICE.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: label!(
            "Shader variant {} pipeline {}",
            compiled_shader,
            pipeline_id
        ),
        layout: Some(pipeline_layout),

        vertex: wgpu::VertexState {
            module: native_shader_module,
            entry_point: None,
            compilation_options: Default::default(),
            buffers: &vertex_state_buffers,
        },
        fragment: Some(wgpu::FragmentState {
            module: native_shader_module,
            entry_point: None,
            compilation_options: Default::default(),
            targets: color_targets,
        }),
        primitive: wgpu::PrimitiveState {
            topology: asset_topology_to_wgpu(topology),
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
        cache: PIPELINE_CACHE.as_ref(),
    });

    Ok(cache::pipeline::insert(pipeline_cache_key, pipeline))
}
