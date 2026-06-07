//! Development overlay tools and API

use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;

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
use wutengine_graphics::mesh::IndexBuffer;
use wutengine_graphics::mesh::VertexBuffer;
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

pub static DEV_OVERLAY: InitOnce<DevOverlayManager> = InitOnce::new();

#[doc(hidden)]
pub fn init() {
    InitOnce::init(&DEV_OVERLAY, DevOverlayManager::new());
}

static EGUI_SHADER: LazyLock<Arc<Shader>> = LazyLock::new(|| {
    let serialized = wutengine_egui::EGUI_SHADER.clone();

    Arc::new(Shader::from_serialized(&serialized).unwrap())
});

struct TextureMaterial {
    texture: Texture,
    sampler: Sampler,
    material: Material,
    cur_screen_size: (f32, f32),
}

pub(crate) struct DevOverlayManager {
    active: AtomicBool,
    egui_context: wutengine_egui::egui::Context,
    textures: Mutex<HashMap<wutengine_egui::egui::TextureId, TextureMaterial>>,
    windows: Mutex<Vec<DevOverlayWindow>>,
}

struct DevOverlayWindow {
    id: DevOverlayWindowId,
    open: bool,
    window: Box<dyn DevelopmentOverlayWindow>,
}

impl DevOverlayManager {
    fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            egui_context: wutengine_egui::egui::Context::default(),
            textures: Mutex::new(HashMap::new()),
            windows: Mutex::new(Vec::new()),
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
}

pub fn render_if_active(
    window: wutengine_input::WindowIdentifier,
    surface: &wgpu::SurfaceTexture,
    scale_factor: f32,
    real_secs_since_start: f64,
) -> Option<wgpu::CommandBuffer> {
    profiling::function_scope!();

    if !is_enabled() {
        return None;
    }

    let surface_format = surface.texture.format().remove_srgb_suffix();
    let surface_view = surface.texture.create_view(&wgpu::TextureViewDescriptor {
        format: Some(surface_format),
        ..Default::default()
    });

    let sfc_size = (surface.texture.size().width, surface.texture.size().height);
    let sfc_points = (
        sfc_size.0 as f32 / scale_factor,
        sfc_size.1 as f32 / scale_factor,
    );

    let egui_input =
        wutengine_egui::gather_input(window, real_secs_since_start, scale_factor, sfc_points);

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

    if let Some((vertex_buffers, index_buffer)) = gather_primitive_buffers(&clipped_output) {
        let mut command_encoder =
            graphics::device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Development overlay command encoder"),
            });

        command_encoder.push_debug_group("Render egui primitives");

        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Development overlay render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &surface_view,
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

        let mut cur_pipeline = None;

        let mut base_vertex = 0;
        let mut base_index = 0;

        render_pass.set_index_buffer(
            index_buffer.raw().slice(..),
            index_buffer.format().to_wgpu(),
        );

        for primitive in clipped_output {
            render_pass.push_debug_group("Render single primitive");

            render_primitive(
                primitive,
                &vertex_buffers,
                &mut base_vertex,
                &mut base_index,
                &mut render_pass,
                &mut texture_map,
                &mut cur_pipeline,
                surface_format,
                sfc_size,
                sfc_points,
                egui_output.pixels_per_point,
            );

            render_pass.pop_debug_group();
        }

        drop(render_pass);
        command_encoder.pop_debug_group();

        free_removed_textures(&mut texture_map, egui_output.textures_delta.free);

        Some(command_encoder.finish())
    } else {
        free_removed_textures(&mut texture_map, egui_output.textures_delta.free);
        None
    }
}

fn gather_primitive_buffers(
    primitives: &[egui::ClippedPrimitive],
) -> Option<(
    HashMap<ShaderVertexAttributeType, VertexBuffer>,
    IndexBuffer,
)> {
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

    let mut vertices = Vec::with_capacity(total_verts);
    let mut uvs = Vec::with_capacity(total_verts);
    let mut colors = Vec::with_capacity(total_verts);
    let mut indices = Vec::with_capacity(total_indices);

    for primitive in primitives {
        let egui::epaint::Primitive::Mesh(mesh) = &primitive.primitive else {
            continue;
        };

        vertices.extend(
            mesh.vertices
                .iter()
                .map(|vtx| GVec3::<f32>::new(vtx.pos.x, vtx.pos.y, 0.0)),
        );
        uvs.extend(
            mesh.vertices
                .iter()
                .map(|vtx| GVec2::<f32>::new(vtx.uv.x, vtx.uv.y)),
        );
        colors.extend(mesh.vertices.iter().map(|vtx| {
            GVec4::<f32>::new(
                vtx.color.r() as f32 / 255.0,
                vtx.color.g() as f32 / 255.0,
                vtx.color.b() as f32 / 255.0,
                vtx.color.a() as f32 / 255.0,
            )
        }));

        indices.extend_from_slice(&mesh.indices);
    }

    let device = graphics::device();

    Some((
        map![
            ShaderVertexAttributeType::Position => VertexBuffer::new(&vertices, ShaderVertexAttributeType::Position, device, false).ok()?,
            ShaderVertexAttributeType::Uv{channel: 0} => VertexBuffer::new(&uvs, ShaderVertexAttributeType::Uv{channel: 0}, device, false).ok()?,
            ShaderVertexAttributeType::Color => VertexBuffer::new(&colors, ShaderVertexAttributeType::Color, device, false).ok()?
        ],
        IndexBuffer::new(&indices, MeshTopology::Triangle, device, false).ok()?,
    ))
}

fn render_primitive(
    primitive: egui::ClippedPrimitive,
    vertex_buffers: &HashMap<ShaderVertexAttributeType, VertexBuffer>,
    base_vertex: &mut u64,
    base_index: &mut u64,
    pass: &mut wgpu::RenderPass,
    texture_map: &mut HashMap<egui::TextureId, TextureMaterial>,
    current_pipeline: &mut Option<Arc<wgpu::RenderPipeline>>,
    surface_format: wgpu::TextureFormat,
    surface_size: (u32, u32),
    surface_points: (f32, f32),
    pixels_per_point: f32,
) {
    profiling::function_scope!();

    match primitive.primitive {
        egui::epaint::Primitive::Mesh(egui_mesh) => {
            let tex_mat = texture_map.get_mut(&egui_mesh.texture_id).unwrap();

            set_surface_size_if_changed(tex_mat, surface_points, wutengine_graphics::queue());

            tex_mat
                .material
                .raw_bind_group_mut()
                .update_bind_group(wutengine_graphics::device());

            let pipeline = graphics::pipeline::get_pipeline(
                &tex_mat.material,
                MeshTopology::Triangle,
                &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::all(),
                })],
            )
            .unwrap();

            if current_pipeline.is_none() || current_pipeline.as_ref().unwrap() != &pipeline {
                pass.set_pipeline(&pipeline);
                *current_pipeline = Some(pipeline);
            }

            let scissor_rect =
                ScissorRect::new(&primitive.clip_rect, pixels_per_point, surface_size);

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
                let Some(vertex_buffer) = vertex_buffers.get(attr_type) else {
                    log::error!(
                        "Mesh is missing vertex buffer for requested attribute: {attr_type}"
                    );
                    return;
                };

                let bytes_per_vtx = graphics::mesh::attr_bytes(*attr_type);
                let start_bytes = (*base_vertex) * bytes_per_vtx as u64;
                let end_bytes = (*base_vertex + num_vertices) * bytes_per_vtx as u64;

                pass.set_vertex_buffer(
                    attr_info.shader_location,
                    vertex_buffer.raw().slice(start_bytes..end_bytes),
                );
            }

            pass.draw_indexed(
                (*base_index as u32)..((*base_index + num_indices) as u32),
                0,
                0..1,
            );

            *base_vertex += num_vertices;
            *base_index += num_indices;
        }
        egui::epaint::Primitive::Callback(_) => unreachable!("Not supported"),
    }
}

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
                let texture = Texture::new(&tex_config_from_egui_data(&delta.image));
                texture.set_data(egui_image_bytes(&delta.image));

                let mut material = Material::new(EGUI_SHADER.clone(), map![]);

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
                        MaterialParameter::Vec2(vec2(
                            surface_points.0 as f32,
                            surface_points.1 as f32,
                        )),
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
