//! [egui] rendering functionality

use alloc::sync::Arc;
use core::num::NonZero;
use std::collections::HashMap;
use wutengine_graphics::label;

use nohash_hasher::IntMap;
use wutengine_assets::assets::mesh::MeshTopology;
use wutengine_assets::assets::shader::ShaderVertexAttributeType;
use wutengine_graphics::mesh::IndexDatatype;
use wutengine_graphics::shader::GVec2;
use wutengine_graphics::shader::GVec3;
use wutengine_graphics::shader::GVec4;
use wutengine_graphics::wgpu;
use wutengine_math::Vec4;
use wutengine_shadercompiler::MATERIAL_PARAMS_BIND_GROUP_INDEX;

use crate::TextureMaterial;
use crate::utils;

/// Writes the given primitives into a new set of buffers, sized by the `_bytes` parameters.
/// Returns the new buffers
pub(crate) fn write_into_new_buffers(
    window_name: &str,
    pos_bytes: u64,
    color_bytes: u64,
    uv_bytes: u64,
    index_bytes: u64,
    primitives: &[egui::ClippedPrimitive],
) -> (
    IntMap<ShaderVertexAttributeType, wgpu::Buffer>,
    wgpu::Buffer,
) {
    profiling::function_scope!();

    let device = wutengine_graphics::device();

    let pos_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: label!("{} Position Buffer", window_name),
        size: pos_bytes,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: true,
    });

    let color_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: label!("{} Color Buffer", window_name),
        size: color_bytes,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: true,
    });

    let uv_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: label!("{} UV Buffer", window_name),
        size: uv_bytes,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: true,
    });

    let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: label!("{} Index Buffer", window_name),
        size: index_bytes,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: true,
    });

    {
        let mut pos_mapped = pos_buf.get_mapped_range_mut(..).unwrap();
        let mut col_mapped = color_buf.get_mapped_range_mut(..).unwrap();
        let mut uv_mapped = uv_buf.get_mapped_range_mut(..).unwrap();
        let mut index_mapped = index_buffer.get_mapped_range_mut(..).unwrap();

        let pos_view = pos_mapped.slice(..);
        let col_view = col_mapped.slice(..);
        let uv_view = uv_mapped.slice(..);
        let index_view = index_mapped.slice(..);

        write_primitives_into_views(pos_view, col_view, uv_view, index_view, primitives);
    }

    pos_buf.unmap();
    color_buf.unmap();
    uv_buf.unmap();
    index_buffer.unmap();

    let mut vertex_bufs = IntMap::default();

    vertex_bufs.insert(ShaderVertexAttributeType::Position, pos_buf);
    vertex_bufs.insert(ShaderVertexAttributeType::Color, color_buf);
    vertex_bufs.insert(ShaderVertexAttributeType::Uv { channel: 0 }, uv_buf);

    (vertex_bufs, index_buffer)
}

/// Writes the given primitives into existing buffers. The amount of bytes that will be written should
/// be given in the `_bytes` parameters
pub(crate) fn write_into_existing_buffers(
    pos_bytes: u64,
    color_bytes: u64,
    uv_bytes: u64,
    index_bytes: u64,
    primitives: &[egui::ClippedPrimitive],
    vertex_buffers: &IntMap<ShaderVertexAttributeType, wgpu::Buffer>,
    index_buffer: &wgpu::Buffer,
) {
    profiling::function_scope!();

    let queue = wutengine_graphics::queue();

    let pos_buf = &vertex_buffers[&ShaderVertexAttributeType::Position];
    let color_buf = &vertex_buffers[&ShaderVertexAttributeType::Color];
    let uv_buf = &vertex_buffers[&ShaderVertexAttributeType::Uv { channel: 0 }];

    let mut pos_write_view = queue
        .write_buffer_with(pos_buf, 0, NonZero::new(pos_bytes).unwrap())
        .unwrap();
    let mut color_write_view = queue
        .write_buffer_with(color_buf, 0, NonZero::new(color_bytes).unwrap())
        .unwrap();
    let mut uv_write_view = queue
        .write_buffer_with(uv_buf, 0, NonZero::new(uv_bytes).unwrap())
        .unwrap();
    let mut index_write_view = queue
        .write_buffer_with(index_buffer, 0, NonZero::new(index_bytes).unwrap())
        .unwrap();

    let pos_view = pos_write_view.slice(..);
    let col_view = color_write_view.slice(..);
    let uv_view = uv_write_view.slice(..);
    let index_view = index_write_view.slice(..);

    write_primitives_into_views(pos_view, col_view, uv_view, index_view, primitives);
}

/// Writes the given primitives into each of the given buffer views
fn write_primitives_into_views(
    mut pos_view: wgpu::WriteOnly<'_, [u8]>,
    mut color_view: wgpu::WriteOnly<'_, [u8]>,
    mut uv_view: wgpu::WriteOnly<'_, [u8]>,
    mut index_view: wgpu::WriteOnly<'_, [u8]>,
    primitives: &[egui::ClippedPrimitive],
) {
    profiling::function_scope!();

    let mut vtx_offset = 0;
    let mut idx_offset = 0;

    let mut pos_staging: Vec<GVec3<f32>> = Vec::new();
    let mut col_staging: Vec<GVec4<f32>> = Vec::new();
    let mut uv_staging: Vec<GVec2<f32>> = Vec::new();

    for primitive in primitives {
        let egui::epaint::Primitive::Mesh(mesh) = &primitive.primitive else {
            continue;
        };

        pos_staging.clear();
        col_staging.clear();
        uv_staging.clear();

        pos_staging.reserve(mesh.vertices.len());
        col_staging.reserve(mesh.vertices.len());
        uv_staging.reserve(mesh.vertices.len());

        // These asserts seem to help the compiler with optimizing
        assert!(
            pos_staging.capacity() - pos_staging.len() >= mesh.vertices.len(),
            "Should have been reserved"
        );
        assert!(
            col_staging.capacity() - col_staging.len() >= mesh.vertices.len(),
            "Should have been reserved"
        );
        assert!(
            uv_staging.capacity() - uv_staging.len() >= mesh.vertices.len(),
            "Should have been reserved"
        );

        for vtx in &mesh.vertices {
            pos_staging.push(GVec3::<f32>::new(vtx.pos.x, vtx.pos.y, 0.0));

            let color_array = vtx.color.to_array();
            const MAP_0_1: f32 = 1.0 / 255.0;
            col_staging.push(GVec4::<f32>::from(
                Vec4::new(
                    color_array[0] as f32,
                    color_array[1] as f32,
                    color_array[2] as f32,
                    color_array[3] as f32,
                ) * MAP_0_1,
            ));

            uv_staging.push(GVec2::<f32>::new(vtx.uv.x, vtx.uv.y));
        }

        let pos_offset = vtx_offset * size_of::<GVec3<f32>>();
        let pos_end = pos_offset + (size_of::<GVec3<f32>>() * mesh.vertices.len());

        let col_offset = vtx_offset * size_of::<GVec4<f32>>();
        let col_end = col_offset + (size_of::<GVec4<f32>>() * mesh.vertices.len());

        let uv_offset = vtx_offset * size_of::<GVec2<f32>>();
        let uv_end = uv_offset + (size_of::<GVec2<f32>>() * mesh.vertices.len());
        pos_view
            .slice(pos_offset..pos_end)
            .copy_from_slice(bytemuck::must_cast_slice(pos_staging.as_slice()));
        color_view
            .slice(col_offset..col_end)
            .copy_from_slice(bytemuck::must_cast_slice(col_staging.as_slice()));
        uv_view
            .slice(uv_offset..uv_end)
            .copy_from_slice(bytemuck::must_cast_slice(uv_staging.as_slice()));

        vtx_offset += mesh.vertices.len();

        let index_bytes = <u32 as IndexDatatype>::as_bytes(&mesh.indices);
        let mut index_slice = index_view.slice(idx_offset..(idx_offset + index_bytes.len()));
        index_slice.copy_from_slice(index_bytes);

        idx_offset += index_bytes.len();
    }
}

/// Rendering helper for the dev overlay.
pub(crate) struct PrimitiveRenderState<'a> {
    /// Surface format of the target
    pub(crate) surface_format: wgpu::TextureFormat,

    /// Vertex buffers to use
    pub(crate) vertex_buffers: &'a IntMap<ShaderVertexAttributeType, wgpu::Buffer>,

    /// Map of materials per texture
    pub(crate) texture_map: &'a mut HashMap<egui::TextureId, TextureMaterial>,

    /// Surface size in pixels
    pub(crate) surface_size: (u32, u32),

    /// Surface size in points
    pub(crate) surface_points: (f32, f32),

    /// Pixels per point
    pub(crate) pixels_per_point: f32,

    /// Current render pipeline. Automatically set and updated to minimize pipeline switches
    pub(crate) cur_pipeline: Option<Arc<wgpu::RenderPipeline>>,

    /// Current base vertex
    pub(crate) base_vertex: u64,

    /// Current base index
    pub(crate) base_index: u64,
}

impl<'a> PrimitiveRenderState<'a> {
    /// Renders a single [egui::ClippedPrimitive]. Primitives should be ordered according to their data in [Self::vertex_buffers] and the currently set index buffer
    pub(crate) fn render_primitive(
        &mut self,
        primitive: egui::ClippedPrimitive,
        pass: &mut wgpu::RenderPass,
    ) {
        match primitive.primitive {
            egui::epaint::Primitive::Mesh(egui_mesh) => {
                let tex_mat = self.texture_map.get_mut(&egui_mesh.texture_id).unwrap();

                tex_mat
                    .set_surface_size_if_changed(self.surface_points, wutengine_graphics::queue());

                tex_mat
                    .material
                    .raw_bind_group_mut()
                    .update_bind_group(wutengine_graphics::device());

                let pipeline = wutengine_graphics::pipeline::get_pipeline(
                    &tex_mat.material,
                    MeshTopology::Triangle,
                    &[Some(wgpu::ColorTargetState {
                        format: self.surface_format,
                        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                )
                .unwrap();

                if self.cur_pipeline.is_none() || self.cur_pipeline.as_ref().unwrap() != &pipeline {
                    pass.set_pipeline(&pipeline);
                    self.cur_pipeline = Some(pipeline);
                }

                let scissor_rect = utils::ScissorRect::new(
                    &primitive.clip_rect,
                    self.pixels_per_point,
                    self.surface_size,
                );

                pass.set_scissor_rect(
                    scissor_rect.x,
                    scissor_rect.y,
                    scissor_rect.width,
                    scissor_rect.height,
                );
                let num_vertices = egui_mesh.vertices.len() as u64;
                let num_indices = egui_mesh.indices.len() as u64;

                pass.set_bind_group(
                    MATERIAL_PARAMS_BIND_GROUP_INDEX,
                    tex_mat.material.raw_bind_group().get_bind_group().unwrap(),
                    &[],
                );

                let attrs = &tex_mat.material.compiled_shader().vertex_attributes;

                for (attr_type, attr_info) in attrs {
                    //TODO: Set this once and use `draw_indexed` with base vertex instead
                    let Some(vertex_buffer) = self.vertex_buffers.get(attr_type) else {
                        log::error!(
                            "Mesh is missing vertex buffer for requested attribute: {attr_type}"
                        );
                        return;
                    };

                    let bytes_per_vtx = wutengine_graphics::mesh::attr_bytes(*attr_type);
                    let start_bytes = (self.base_vertex) * bytes_per_vtx as u64;
                    let end_bytes = (self.base_vertex + num_vertices) * bytes_per_vtx as u64;

                    pass.set_vertex_buffer(
                        attr_info.shader_location,
                        vertex_buffer.slice(start_bytes..end_bytes),
                    );
                }

                pass.draw_indexed(
                    (self.base_index as u32)..((self.base_index + num_indices) as u32),
                    0,
                    0..1,
                );

                self.base_vertex += num_vertices;
                self.base_index += num_indices;
            }
            egui::epaint::Primitive::Callback(_) => unreachable!("Not supported"),
        }
    }
}
