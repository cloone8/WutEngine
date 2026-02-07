//! Graphics pipeline functions

use std::sync::Arc;

use smallvec::SmallVec;
use wutengine_util_macro::unique_id_type64;

use crate::graphics::{self, GFX_DEVICE};

use super::cache;
use super::cache::pipeline::PipelineCacheKey;
use super::material::NativeMaterial;

unique_id_type64! {
    /// Unique ID for a render pipeline. Mostly used for debug labels
    PipelineId
}

pub(crate) fn get_pipeline(
    material: &NativeMaterial,
    color_targets: &[Option<wgpu::ColorTargetState>],
) -> Arc<wgpu::RenderPipeline> {
    profiling::function_scope!();

    let pipeline_cache_key = PipelineCacheKey {
        shader: material.compiled_shader_id(),
        color_targets: color_targets.into(),
    };

    if let Some(cached_pipeline) = cache::pipeline::find(&pipeline_cache_key) {
        return cached_pipeline;
    }

    let pipeline_id = PipelineId::new();

    log::debug!(
        "Creating new pipeline with ID {pipeline_id} for shader variant {}",
        material.compiled_shader_id()
    );

    let compiled_shader = graphics::shader::compile(&material.shader, &material.get_keywords());

    let pipeline_layout = &compiled_shader.pipeline_layout;

    let native_shader_module = &compiled_shader.module;

    let pipeline = GFX_DEVICE.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(
            format!(
                "Shader {} variant {} pipeline {}",
                material.shader.name,
                material.compiled_shader_id(),
                pipeline_id
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
            topology: wgpu::PrimitiveTopology::TriangleList,
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

    cache::pipeline::insert(pipeline_cache_key, pipeline)
}
