#![doc = include_str!("../README.md")]

extern crate alloc;

use alloc::sync::Arc;
use core::num::NonZero;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use nohash_hasher::IntMap;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;
use wutengine_egui::egui::ClippedPrimitive;
use wutengine_graphics::mesh::IndexDatatype;
use wutengine_math::Vec4;

use wutengine_asset::Asset;
use wutengine_asset::AssetHandle;
use wutengine_asset::assets::mesh::MeshTopology;
use wutengine_asset::assets::shader::ShaderVertexAttributeType;
use wutengine_egui::ScissorRect;
use wutengine_egui::egui;
use wutengine_egui::egui_image_bytes;
use wutengine_egui::sampler_from_egui;
use wutengine_egui::tex_config_from_egui_data;
use wutengine_graphics::shader::Shader;
use wutengine_shadercompiler::MATERIAL_PARAMS_BIND_GROUP_INDEX;
use wutengine_util_macro::unique_id_type32;

use wutengine_graphics as graphics;
use wutengine_graphics::material::Material;
use wutengine_graphics::material::MaterialParameter;
use wutengine_graphics::sampler::Sampler;
use wutengine_graphics::shader::GVec2;
use wutengine_graphics::shader::GVec3;
use wutengine_graphics::shader::GVec4;
use wutengine_graphics::texture::Texture;
use wutengine_graphics::wgpu;

use wutengine_math::vec2;
use wutengine_util::InitOnce;
use wutengine_util::map;

#[doc(inline)]
pub use wutengine_egui;

unique_id_type32! {
    DevOverlayWindowId
}

/// Global [DevOverlayManager]
static DEV_OVERLAY: InitOnce<DevOverlayManager> = InitOnce::new();

#[doc(hidden)]
pub fn init() {
    InitOnce::init(&DEV_OVERLAY, DevOverlayManager::new());
}

/// Shader for [egui]
static EGUI_SHADER: LazyLock<Arc<Shader>> = LazyLock::new(|| {
    let serialized = wutengine_egui::EGUI_SHADER.clone();

    Arc::new(Shader::from_serialized(&serialized).unwrap())
});

/// A material for rendering one texture. All [egui] meshes use the same shader,
/// so we can just create a different material per texture to reduce bindgroup updates
struct TextureMaterial {
    /// The texture this material is for
    texture: Texture,

    /// The sampler used to sample [Self::texture]
    sampler: Sampler,

    /// The actual material
    material: Material,

    /// The last screen size set on [Self::material]
    cur_screen_size: (f32, f32),
}

/// Manages the calculating and rendering of the development overlay
pub(crate) struct DevOverlayManager {
    /// Whether the overlay should be active
    active: AtomicBool,

    /// The [egui::Context]
    egui_context: egui::Context,

    /// The materials for each texture
    textures: Mutex<HashMap<egui::TextureId, TextureMaterial>>,

    /// All registered dev windows
    windows: Mutex<Vec<DevOverlayWindow>>,

    /// The GPU buffers containing all mesh vertices and indices
    buffers: Mutex<
        Option<(
            IntMap<ShaderVertexAttributeType, wgpu::Buffer>,
            wgpu::Buffer,
        )>,
    >,

    /// The calculated drawing parameters
    to_draw: Mutex<Option<DevOverlayDrawable>>,
}

/// Drawing parameters for use by [render_overlay], calculated by [run_overlay_logic]
struct DevOverlayDrawable {
    /// The primitives to draw
    primitives: Vec<ClippedPrimitive>,

    /// Textures to free after drawing
    to_free: Vec<egui::TextureId>,

    /// The pixels per point to use
    pixels_per_point: f32,
}

/// A single development overlay window, injected through [add_development_overlay_window]
struct DevOverlayWindow {
    /// The unique ID of the window
    id: DevOverlayWindowId,

    /// Whether the window should be open now
    open: bool,

    /// The implementation of the window
    window: Box<dyn DevelopmentOverlayWindow>,
}

impl DevOverlayManager {
    /// A new empty [DevOverlayManager]
    fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            egui_context: wutengine_egui::egui::Context::default(),
            textures: Mutex::new(HashMap::new()),
            windows: Mutex::new(Vec::new()),
            buffers: Mutex::new(None),
            to_draw: Mutex::new(None),
        }
    }
}

/// A WutEngine development overlay. Can be added to the engine using [crate::development_overlay::add_development_overlay_window]
pub trait DevelopmentOverlayWindow: Send + Sync + 'static {
    /// The name of the overlay
    fn name(&self) -> &str;

    /// An icon to show on the overlay window. Optional. Should be an emoji or something similar
    fn icon(&self) -> Option<&str> {
        None
    }

    /// Shows the UI
    fn show(&mut self, ui: &mut egui::Ui);

    /// Called when the window was either opened or closed
    fn window_state_changed(&mut self, opened: bool) {
        _ = opened;
    }
}

/// Runs the logic to draw the dev overlay, if it is active.
///
/// Returns a [std::sync::mpsc::Receiver] that will receive exactly one message when the overlay is done
/// calculating, as that is done on a different thread. When the receiver has received its message, [render_overlay] should
/// be called before another call to [run_overlay_logic] in order to render the calculated overlay
/// onto a target
pub fn run_overlay_logic(
    input_window: wutengine_input::WindowIdentifier,
    window_info: wutengine_egui::EguiWindowInfo,
    surface_size: (u32, u32),
    scale_factor: f32,
    real_secs_since_start: f64,
) -> std::sync::mpsc::Receiver<()> {
    profiling::function_scope!();

    let (is_done_send, is_done_recv) = std::sync::mpsc::sync_channel(1);

    if !is_enabled() {
        is_done_send.send(()).unwrap();
        return is_done_recv;
    }

    rayon::spawn(move || {
        profiling::scope!("Run overlay logic");

        let sfc_size = (surface_size.0, surface_size.1);
        let sfc_points = (
            sfc_size.0 as f32 / scale_factor,
            sfc_size.1 as f32 / scale_factor,
        );

        let egui_input = wutengine_egui::gather_input(
            input_window,
            &window_info,
            graphics::active_config().limits.max_texture_dimension_2d as usize,
            real_secs_since_start,
            scale_factor,
            sfc_points,
        );

        let mut windows = DEV_OVERLAY.windows.lock().unwrap();

        let egui_output = DEV_OVERLAY.egui_context.run_ui(egui_input.clone(), |ui| {
            egui::Window::new("WutEngine Development Overlay")
                .collapsible(false)
                .order(egui::Order::Background)
                .resizable(false)
                .default_open(true)
                .show(ui, |ui| {
                    if windows.is_empty() {
                        ui.label("No development windows registered");
                        return;
                    }

                    for window in windows.iter_mut() {
                        if ui.button(window.window.name()).clicked() {
                            window.open = !window.open;
                            window.window.window_state_changed(window.open);
                        }

                        let title_with_icon = window
                            .window
                            .icon()
                            .map(|icon| format!("{} {}", icon, window.window.name()));

                        let title_str = match &title_with_icon {
                            Some(with_icon) => with_icon.as_str(),
                            None => window.window.name(),
                        };

                        egui::Window::new(title_str)
                            .id(egui::Id::new(window.id))
                            .open(&mut window.open)
                            .show(ui, |ui| {
                                window.window.show(ui);
                            });
                    }
                });
        });

        let clipped_output = DEV_OVERLAY
            .egui_context
            .tessellate(egui_output.shapes, egui_output.pixels_per_point);

        log::trace!("egui returned {} primitives", clipped_output.len());
        log::trace!(
            "egui returned {} new textures and wants to free {} textures",
            egui_output.textures_delta.set.len(),
            egui_output.textures_delta.free.len()
        );

        let mut texture_map = DEV_OVERLAY.textures.lock().unwrap();

        upload_new_textures(&mut texture_map, egui_output.textures_delta.set, sfc_points);

        let mut buffers = DEV_OVERLAY.buffers.lock().unwrap();

        gather_primitive_buffers(&clipped_output, &mut buffers);

        if buffers.is_some() {
            *DEV_OVERLAY.to_draw.lock().unwrap() = Some(DevOverlayDrawable {
                primitives: clipped_output,
                to_free: egui_output.textures_delta.free,
                pixels_per_point: egui_output.pixels_per_point,
            });
        }

        is_done_send.send(()).unwrap();
    });

    is_done_recv
}

/// Renders the current development overlay. Should be preceded by a call to [run_overlay_logic], and the returned channel should
/// have been waited on.
pub fn render_overlay(target: &wgpu::Texture, command_encoder: &mut wgpu::CommandEncoder) -> bool {
    let Some(drawable) = DEV_OVERLAY.to_draw.lock().unwrap().take() else {
        return false;
    };

    profiling::function_scope!();

    let target_format = target.format().remove_srgb_suffix();
    let target_view = target.create_view(&wgpu::TextureViewDescriptor {
        format: Some(target_format),
        ..Default::default()
    });

    let tgt_size = (target.size().width, target.size().height);
    let tgt_points = (
        tgt_size.0 as f32 / drawable.pixels_per_point,
        tgt_size.1 as f32 / drawable.pixels_per_point,
    );

    let mut texture_map = DEV_OVERLAY.textures.lock().unwrap();
    let buffers = DEV_OVERLAY.buffers.lock().unwrap();

    let Some((vertex_buffers, index_buffer)) = &*buffers else {
        // Error. Just do do cleanup
        free_removed_textures(&mut texture_map, drawable.to_free);
        return false;
    };

    command_encoder.push_debug_group("Render egui primitives");

    let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Development overlay render pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &target_view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

    let mut renderstate = PrimitiveRenderState {
        surface_format: target_format,
        vertex_buffers,
        texture_map: &mut texture_map,
        surface_size: tgt_size,
        surface_points: tgt_points,
        pixels_per_point: drawable.pixels_per_point,
        cur_pipeline: None,
        base_vertex: 0,
        base_index: 0,
    };

    for primitive in drawable.primitives {
        render_pass.push_debug_group("Render single primitive");

        renderstate.render_primitive(primitive, &mut render_pass);

        render_pass.pop_debug_group();
    }

    drop(render_pass);
    command_encoder.pop_debug_group();

    free_removed_textures(&mut texture_map, drawable.to_free);

    true
}

/// Rendering helper for the dev overlay.
struct PrimitiveRenderState<'a> {
    /// Surface format of the target
    surface_format: wgpu::TextureFormat,

    /// Vertex buffers to use
    vertex_buffers: &'a IntMap<ShaderVertexAttributeType, wgpu::Buffer>,

    /// Map of materials per texture
    texture_map: &'a mut HashMap<egui::TextureId, TextureMaterial>,

    /// Surface size in pixels
    surface_size: (u32, u32),

    /// Surface size in points
    surface_points: (f32, f32),

    /// Pixels per point
    pixels_per_point: f32,

    /// Current render pipeline. Automatically set and updated to minimize pipeline switches
    cur_pipeline: Option<Arc<wgpu::RenderPipeline>>,

    /// Current base vertex
    base_vertex: u64,

    /// Current base index
    base_index: u64,
}

impl<'a> PrimitiveRenderState<'a> {
    /// Renders a single [egui::ClippedPrimitive]. Primitives should be ordered according to their data in [Self::vertex_buffers] and the currently set index buffer
    fn render_primitive(&mut self, primitive: egui::ClippedPrimitive, pass: &mut wgpu::RenderPass) {
        match primitive.primitive {
            egui::epaint::Primitive::Mesh(egui_mesh) => {
                let tex_mat = self.texture_map.get_mut(&egui_mesh.texture_id).unwrap();

                set_surface_size_if_changed(
                    tex_mat,
                    self.surface_points,
                    wutengine_graphics::queue(),
                );

                tex_mat
                    .material
                    .raw_bind_group_mut()
                    .update_bind_group(wutengine_graphics::device());

                let pipeline = graphics::pipeline::get_pipeline(
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

                let scissor_rect = ScissorRect::new(
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

                    let bytes_per_vtx = graphics::mesh::attr_bytes(*attr_type);
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

/// Gathers the data for all given primitives into the buffers given in `buffers`. If the buffers
/// do not yet exist or are not large enough, they will be replaced with appropriately sized buffers
fn gather_primitive_buffers(
    primitives: &[egui::ClippedPrimitive],
    buffers: &mut Option<(
        IntMap<ShaderVertexAttributeType, wgpu::Buffer>,
        wgpu::Buffer,
    )>,
) {
    profiling::function_scope!();

    let mut total_verts = 0;
    let mut total_indices = 0;

    for primitive in primitives {
        let egui::epaint::Primitive::Mesh(mesh) = &primitive.primitive else {
            continue;
        };

        total_verts += mesh.vertices.len();
        total_indices += mesh.indices.len();
    }

    if total_verts == 0 || total_indices == 0 {
        return;
    }

    let recreate_buffers = match buffers {
        Some((vert_bufs, idx_buf)) => {
            ((idx_buf.size() as usize) / size_of::<u32>()) < total_indices
                || (vert_bufs[&ShaderVertexAttributeType::Position].size() as usize)
                    / size_of::<GVec3<f32>>()
                    < total_verts
        }
        None => true,
    };

    if recreate_buffers {
        *buffers = None;
    }

    let pos_bytes =
        (graphics::mesh::attr_bytes(ShaderVertexAttributeType::Position) * total_verts) as u64;
    let color_bytes =
        (graphics::mesh::attr_bytes(ShaderVertexAttributeType::Color) * total_verts) as u64;
    let uv_bytes = (graphics::mesh::attr_bytes(ShaderVertexAttributeType::Uv { channel: 0 })
        * total_verts) as u64;
    let index_bytes = (size_of::<u32>() * total_indices) as u64;

    match buffers {
        Some((vertex_buffers, index_buffer)) => {
            write_into_existing_buffers(
                pos_bytes,
                color_bytes,
                uv_bytes,
                index_bytes,
                primitives,
                vertex_buffers,
                index_buffer,
            );
        }
        None => {
            *buffers = Some(write_into_new_buffers(
                pos_bytes,
                color_bytes,
                uv_bytes,
                index_bytes,
                primitives,
            ));
        }
    }
}

/// Writes the given primitives into a new set of buffers, sized by the `_bytes` parameters.
/// Returns the new buffers
fn write_into_new_buffers(
    pos_bytes: u64,
    color_bytes: u64,
    uv_bytes: u64,
    index_bytes: u64,
    primitives: &[egui::ClippedPrimitive],
) -> (
    IntMap<ShaderVertexAttributeType, wgpu::Buffer>,
    wgpu::Buffer,
) {
    let device = graphics::device();

    let pos_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Development Overlay Position Buffer"),
        size: pos_bytes,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: true,
    });

    let color_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Development Overlay Color Buffer"),
        size: color_bytes,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: true,
    });

    let uv_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Development Overlay UV Buffer"),
        size: uv_bytes,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: true,
    });

    let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Development Overlay Index Buffer"),
        size: index_bytes,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: true,
    });

    {
        let mut pos_mapped = pos_buf.get_mapped_range_mut(..);
        let mut col_mapped = color_buf.get_mapped_range_mut(..);
        let mut uv_mapped = uv_buf.get_mapped_range_mut(..);
        let mut index_mapped = index_buffer.get_mapped_range_mut(..);

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
fn write_into_existing_buffers(
    pos_bytes: u64,
    color_bytes: u64,
    uv_bytes: u64,
    index_bytes: u64,
    primitives: &[egui::ClippedPrimitive],
    vertex_buffers: &IntMap<ShaderVertexAttributeType, wgpu::Buffer>,
    index_buffer: &wgpu::Buffer,
) {
    let queue = graphics::queue();

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

/// Sets the `screen_size` parameter on the given texture material, if the screen size
/// has changed
fn set_surface_size_if_changed(
    texmat: &mut TextureMaterial,
    sfc_size: (f32, f32),
    queue: &wgpu::Queue,
) {
    if texmat.cur_screen_size == sfc_size {
        return;
    }

    texmat
        .material
        .raw_bind_group_mut()
        .set_parameter(
            "screen_size",
            MaterialParameter::Vec2(vec2(sfc_size.0, sfc_size.1)),
            queue,
        )
        .unwrap();

    texmat.cur_screen_size = sfc_size;
}

/// Uploads new texture materials (or updates existing ones) depending on the parameters in `texture_map`
fn upload_new_textures(
    texture_map: &mut HashMap<egui::TextureId, TextureMaterial>,
    to_set: Vec<(egui::TextureId, egui::epaint::ImageDelta)>,
    surface_points: (f32, f32),
) {
    profiling::function_scope!();

    let queue = graphics::queue();
    let device = graphics::device();

    for (tex_id, delta) in to_set {
        let sampler = Sampler::from_serialized(&sampler_from_egui(&delta.options)).unwrap();

        match delta.pos {
            Some(pos) => {
                let texmat = texture_map.get_mut(&tex_id).unwrap();

                texmat.sampler = sampler;

                texmat
                    .material
                    .raw_bind_group_mut()
                    .set_parameter(
                        "ui_texture_sampler",
                        MaterialParameter::Sampler(AssetHandle::new(texmat.sampler.clone())),
                        queue,
                    )
                    .unwrap();

                set_surface_size_if_changed(texmat, surface_points, queue);

                texmat
                    .material
                    .raw_bind_group_mut()
                    .update_bind_group(device);

                texmat.texture.set_partial_data(
                    egui_image_bytes(&delta.image),
                    wgpu::Origin3d {
                        x: pos[0] as u32,
                        y: pos[1] as u32,
                        z: 0,
                    },
                    wgpu::Extent3d {
                        width: delta.image.width() as u32,
                        height: delta.image.height() as u32,
                        depth_or_array_layers: 1,
                    },
                );
            }
            None => {
                let texture = Texture::new(&tex_config_from_egui_data(&delta.image), 1);
                texture.set_data(egui_image_bytes(&delta.image));

                let mut material = Material::new(EGUI_SHADER.clone(), map!["DITHERING" => 0u64]);

                material
                    .raw_bind_group_mut()
                    .set_parameter(
                        "ui_texture_sampler",
                        MaterialParameter::Sampler(AssetHandle::new(sampler.clone())),
                        queue,
                    )
                    .unwrap();

                material
                    .raw_bind_group_mut()
                    .set_parameter(
                        "ui_texture",
                        MaterialParameter::Texture2D(AssetHandle::new(texture.clone())),
                        queue,
                    )
                    .unwrap();

                material
                    .raw_bind_group_mut()
                    .set_parameter(
                        "screen_size",
                        MaterialParameter::Vec2(vec2(surface_points.0, surface_points.1)),
                        queue,
                    )
                    .unwrap();

                material.raw_bind_group_mut().update_bind_group(device);

                texture_map.insert(
                    tex_id,
                    TextureMaterial {
                        texture,
                        sampler,
                        material,
                        cur_screen_size: surface_points,
                    },
                );
            }
        }
    }
}

/// Removes the [TextureMaterial] belonging to any texutre in `to_remove`
fn free_removed_textures(
    texture_map: &mut HashMap<egui::TextureId, TextureMaterial>,
    to_remove: Vec<egui::TextureId>,
) {
    profiling::function_scope!();

    for tex_id in to_remove {
        texture_map.remove(&tex_id);
    }
}

/// Enable the development overlay
#[inline]
pub fn enable() {
    set_state(true);
}

/// Disable the development overlay
#[inline]
pub fn disable() {
    set_state(false);
}

/// Enable or disable the development overlay
pub fn set_state(active: bool) {
    DEV_OVERLAY.active.store(active, Ordering::Release);
}

/// Returns whether the development overlay is currently enabled
pub fn is_enabled() -> bool {
    DEV_OVERLAY.active.load(Ordering::Acquire)
}

/// Add a new [DevelopmentOverlayWindow] to the engine
pub fn add_development_overlay_window<T: DevelopmentOverlayWindow>(window: T) {
    DEV_OVERLAY.windows.lock().unwrap().push(DevOverlayWindow {
        id: DevOverlayWindowId::new(),
        open: false,
        window: Box::new(window),
    });
}
