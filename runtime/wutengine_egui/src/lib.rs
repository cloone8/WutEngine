#![doc = include_str!("../README.md")]

extern crate alloc;

use alloc::sync::Arc;
use render::PrimitiveRenderState;
use std::assert_matches;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;
use wutengine_assets::FromSerializedAsset;
use wutengine_graphics::label;
use wutengine_util::error_once;
use wutengine_util::warn_once;

use nohash_hasher::IntMap;
use wutengine_assets::assets::shader::SerializedShader;
use wutengine_assets::assets::shader::ShaderSource;
use wutengine_assets::assets::shader::ShaderVertexAttributeType;
use wutengine_graphics::material::Material;
use wutengine_graphics::material::MaterialParameter;
use wutengine_graphics::sampler::Sampler;
use wutengine_graphics::shader::GVec3;
use wutengine_graphics::shader::Shader;
use wutengine_graphics::texture::Texture;
use wutengine_graphics::wgpu;
use wutengine_input::WindowIdentifier;
use wutengine_math::vec2;
use wutengine_util::map;

mod input;
mod key_mapping;
mod render;
pub mod utils;

pub use key_mapping::*;

pub use egui;

/// Information for the window we're rendering [`egui`] on
#[derive(Debug, Clone, Copy)]
pub struct EguiWindowInfo {
    /// Window is in focus
    pub focused: bool,

    /// Window is occluded
    pub occluded: bool,

    /// Window is minimized
    pub minimized: bool,

    /// Window is maximized
    pub maximized: bool,
}

impl Default for EguiWindowInfo {
    fn default() -> Self {
        Self {
            focused: true,
            occluded: false,
            minimized: false,
            maximized: false,
        }
    }
}

/// Shader for [`egui`]
pub static EGUI_SHADER: LazyLock<Arc<Shader>> = LazyLock::new(|| {
    let descriptor = include_str!("egui.json");
    let source = include_str!("egui.wgsl");

    let mut shader =
        serde_json::from_str::<SerializedShader>(descriptor).expect("Could not get egui shader");

    shader.source = ShaderSource::Inline {
        content: source.to_owned(),
    };

    Arc::new(Shader::from_serialized_asset(shader).unwrap())
});

/// Information on a single egui viewport
#[derive(derive_more::Debug)]
pub struct EguiWindow {
    /// The title
    pub title: String,

    /// The window identifier (for input)
    pub input_window_identifier: WindowIdentifier,

    /// General window info
    pub window_info: EguiWindowInfo,

    /// Texture size limit
    pub tex2d_size_limit: usize,

    /// Surface size in logical points
    pub surface_size_points: (f32, f32),

    /// Surface scale factor
    pub scale_factor: f32,

    /// GPU buffers containing vertex and index data for this window
    gpu_buffers: Mutex<
        Option<(
            IntMap<ShaderVertexAttributeType, wgpu::Buffer>,
            wgpu::Buffer,
        )>,
    >,

    /// Last calculated output by [`Self::run_logic`]
    last_output: Mutex<Option<WindowDrawable>>,
}

/// Drawing parameters
#[derive(Debug)]
struct WindowDrawable {
    /// The primitives to draw
    primitives: Vec<egui::ClippedPrimitive>,

    /// Textures to free after drawing
    to_free: Vec<egui::TextureId>,

    /// The pixels per point to use
    pixels_per_point: f32,
}

impl EguiWindow {
    /// Creates a new egui window taking input from the given [`WindowIdentifier`] and displaying the provided UI callback, with the given
    /// initial size.
    pub fn new(
        input_window_identifier: WindowIdentifier,
        surface_size_points: (f32, f32),
    ) -> Box<Self> {
        Box::new(Self {
            title: "WutEngine".to_string(),
            input_window_identifier,
            window_info: EguiWindowInfo::default(),
            tex2d_size_limit: wutengine_graphics::active_config()
                .limits
                .max_texture_dimension_2d as usize,
            surface_size_points,
            scale_factor: 1.0,
            gpu_buffers: Mutex::new(None),
            last_output: Mutex::new(None),
        })
    }

    /// Returns the input required to run [`egui`] for a frame
    fn gather_input(&self, real_time_secs: f64) -> egui::RawInput {
        profiling::function_scope!();

        let sfc_rect = egui::Rect {
            min: egui::Pos2::ZERO,
            max: egui::Pos2::new(self.surface_size_points.0, self.surface_size_points.1),
        };

        let mut egui_events = Vec::new();
        //TODO: Emit egui window focus event if focus changed

        let modifiers = input::add_keyboard_events(&mut egui_events);

        input::add_mouse_events(
            self.input_window_identifier,
            modifiers,
            self.scale_factor,
            &mut egui_events,
        );

        if !egui_events.is_empty() {
            log::trace!("Sending events: {egui_events:#?}");
        }

        egui::RawInput {
            viewport_id: egui::ViewportId::ROOT,
            viewports: map![
                egui::ViewportId::ROOT => egui::ViewportInfo {
                    parent: None,
                    title: Some(self.title.clone()),
                    events: vec![],
                    native_pixels_per_point: Some(self.scale_factor),
                    monitor_size: None,
                    inner_rect: Some(sfc_rect),
                    outer_rect: None,
                    minimized: Some(self.window_info.minimized),
                    maximized: Some(self.window_info.maximized),
                    fullscreen: None,
                    focused: Some(self.window_info.focused),
                    occluded: Some(self.window_info.occluded)
                }
            ],
            safe_area_insets: None,
            screen_rect: Some(sfc_rect),
            max_texture_side: Some(self.tex2d_size_limit),
            time: Some(real_time_secs),
            predicted_dt: 1.0 / 60.0,
            modifiers,
            events: egui_events,
            hovered_files: vec![],
            dropped_files: vec![],
            focused: self.window_info.focused,
            system_theme: None,
        }
    }

    /// Handle platform output from egui
    fn handle_platform_output(output: &egui::PlatformOutput) -> LogicOutput {
        profiling::function_scope!();

        let mut logic_output = LogicOutput::default();

        for command in &output.commands {
            match command {
                egui::OutputCommand::CopyText(_) => {
                    warn_once!("Copy text output not yet supported");
                }
                egui::OutputCommand::CopyImage(_) => {
                    warn_once!("Copy image output not yet supported");
                }
                egui::OutputCommand::OpenUrl(_) => {
                    error_once!("Open URL output not yet supported");
                }
            }
        }

        if output.cursor_icon == egui::CursorIcon::None {
            logic_output.cursor = cursor_icon::CursorIcon::Default;
            logic_output.cursor_visible = false;
        } else {
            logic_output.cursor = utils::cursor_icon_from_egui(output.cursor_icon)
                .expect("Cursor should not be hidden");
            logic_output.cursor_visible = true;
        }

        if output.cursor_image.is_some() {
            warn_once!("Cursor images not yet supported");
        }

        if output.requested_discard() {
            warn_once!("Discards not yet supported");
        }

        logic_output
    }

    /// Runs the egui UI logic on the provided context, with the provided texture map.
    ///
    /// Should be run exactly once before calling [`Self::render_window`]
    ///
    /// TODO: Combine `context` and `texture_map` into one struct
    pub fn run_logic(
        &self,
        context: &egui::Context,
        texture_map: &TextureMaterialMap,
        ui_callback: impl FnMut(&mut egui::Ui),
    ) -> LogicOutput {
        profiling::function_scope!();

        let real_time = wutengine_time::unscaled_time64();
        let egui_input = self.gather_input(real_time);

        let egui_output = context.run_ui(egui_input, ui_callback);

        let logic_output = Self::handle_platform_output(&egui_output.platform_output);

        let clipped_output = context.tessellate(egui_output.shapes, egui_output.pixels_per_point);

        log::trace!("egui returned {} primitives", clipped_output.len());
        log::trace!(
            "egui returned {} new textures and wants to free {} textures",
            egui_output.textures_delta.set.len(),
            egui_output.textures_delta.free.len()
        );

        // Pick any arbitrary initial window size, because it'll be update automatically at render time anyway
        texture_map.upload_new(&egui_output.textures_delta.set, (100.0, 100.0));

        self.gather_primitive_buffers(&clipped_output);

        *self.last_output.lock().unwrap() = Some(WindowDrawable {
            primitives: clipped_output,
            to_free: egui_output.textures_delta.free, //TODO: If `last_output` is not None, we leak textures
            pixels_per_point: egui_output.pixels_per_point,
        });

        logic_output
    }

    /// Renders the calculated UI onto the provided target texture.
    /// Writes the textures that should be freed into `to_free`
    ///
    /// Should be run after [`Self::run_logic`]
    pub fn render_window(
        &self,
        target: &wgpu::Texture,
        texture_map: &TextureMaterialMap,
        command_encoder: &mut wgpu::CommandEncoder,
        to_free: &mut Vec<egui::TextureId>,
    ) {
        profiling::function_scope!();

        debug_assert!(
            target
                .usage()
                .contains(wgpu::TextureUsages::RENDER_ATTACHMENT),
            "Cannot render to given texture"
        );

        let Some(drawable) = self.last_output.lock().unwrap().take() else {
            return;
        };

        let target_format = target.format().remove_srgb_suffix();
        let target_view = target.create_view(&wgpu::TextureViewDescriptor {
            format: Some(target_format),
            ..Default::default()
        });

        let tgt_size = (target.size().width, target.size().height);

        #[expect(clippy::cast_precision_loss, reason = "Expected here")]
        let tgt_points = (
            tgt_size.0 as f32 / drawable.pixels_per_point,
            tgt_size.1 as f32 / drawable.pixels_per_point,
        );

        let buffers = self.gpu_buffers.lock().unwrap();

        let Some((vertex_buffers, index_buffer)) = &*buffers else {
            // Error. Just do do cleanup
            to_free.extend(drawable.to_free);
            return;
        };

        // Create new materials

        command_encoder.push_debug_group("Render egui primitives");

        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: label!("EguiWindow \"{}\" render pass", self.title),
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

        let mut locked_texmap = texture_map.0.lock().unwrap();
        let mut renderstate = PrimitiveRenderState {
            surface_format: target_format,
            vertex_buffers,
            texture_map: &mut locked_texmap,
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

        drop(locked_texmap);

        to_free.extend(drawable.to_free);
    }

    /// Gathers the data for all given primitives into the buffers given in `buffers`. If the buffers
    /// do not yet exist or are not large enough, they will be replaced with appropriately sized buffers
    fn gather_primitive_buffers(&self, primitives: &[egui::ClippedPrimitive]) {
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

        let mut buffers_lock = self.gpu_buffers.lock().unwrap();
        let buffers: &mut Option<_> = &mut buffers_lock;

        let recreate_buffers = match buffers {
            Some((vert_bufs, idx_buf)) => {
                let idx_buf_size = usize::try_from(idx_buf.size()).unwrap();
                let vtx_buf_size =
                    usize::try_from(vert_bufs[&ShaderVertexAttributeType::Position].size())
                        .unwrap();

                (idx_buf_size / size_of::<u32>()) < total_indices
                    || (vtx_buf_size / size_of::<GVec3<f32>>()) < total_verts
            }
            None => true,
        };

        if recreate_buffers {
            *buffers = None;
        }

        let pos_bytes = (wutengine_graphics::mesh::attr_bytes(ShaderVertexAttributeType::Position)
            * total_verts) as u64;
        let color_bytes = (wutengine_graphics::mesh::attr_bytes(ShaderVertexAttributeType::Color)
            * total_verts) as u64;
        let uv_bytes =
            (wutengine_graphics::mesh::attr_bytes(ShaderVertexAttributeType::Uv { channel: 0 })
                * total_verts) as u64;
        let index_bytes = (size_of::<u32>() * total_indices) as u64;

        match buffers {
            Some((vertex_buffers, index_buffer)) => {
                render::write_into_existing_buffers(
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
                *buffers = Some(render::write_into_new_buffers(
                    &self.title,
                    pos_bytes,
                    color_bytes,
                    uv_bytes,
                    index_bytes,
                    primitives,
                ));
            }
        }
    }
}

/// Logical output that needs to be handled by the caller
#[derive(Debug, Clone, Default)]
pub struct LogicOutput {
    /// The requested cursor
    pub cursor: cursor_icon::CursorIcon,

    /// Whether the cursor is visible or not
    pub cursor_visible: bool,
}

/// A map of egui textures to WutEngine materials. Used by [`EguiWindow`]
#[derive(Debug, Default)]
pub struct TextureMaterialMap(Mutex<HashMap<egui::TextureId, TextureMaterial>>);

impl TextureMaterialMap {
    /// Uploads new textures (and updates existing ones) into the map, with the given initial surface size in points
    fn upload_new(
        &self,
        set: &[(egui::TextureId, egui::epaint::image::ImageDelta)],
        surface_points: (f32, f32),
    ) {
        profiling::function_scope!();

        let queue = wutengine_graphics::queue();
        let device = wutengine_graphics::device();
        let mut texture_map = self.0.lock().unwrap();

        for (tex_id, delta) in set {
            assert_matches!(
                tex_id,
                egui::TextureId::Managed(_),
                "Only managed textures are supported at the moment"
            );

            let sampler = Arc::new(
                Sampler::from_serialized_asset(utils::sampler_from_egui(delta.options)).unwrap(),
            );

            if let Some(pos) = delta.pos {
                // Update subregion of texture

                let texmat = texture_map.get_mut(tex_id).unwrap();

                texmat.sampler = sampler;

                texmat
                    .material
                    .raw_bind_group_mut()
                    .set_parameter(
                        "ui_texture_sampler",
                        MaterialParameter::Sampler(texmat.sampler.clone()),
                        queue,
                    )
                    .unwrap();

                texmat.set_surface_size_if_changed(surface_points, queue);

                texmat
                    .material
                    .raw_bind_group_mut()
                    .update_bind_group(device);

                texmat.texture.set_partial_data(
                    utils::egui_image_bytes(&delta.image),
                    wgpu::Origin3d {
                        x: u32::try_from(pos[0]).unwrap(),
                        y: u32::try_from(pos[1]).unwrap(),
                        z: 0,
                    },
                    wgpu::Extent3d {
                        width: u32::try_from(delta.image.width()).unwrap(),
                        height: u32::try_from(delta.image.height()).unwrap(),
                        depth_or_array_layers: 1,
                    },
                );
            } else {
                // Update entire texture

                let texture = Arc::new(Texture::new(
                    &utils::tex_config_from_egui_data(&delta.image),
                    1,
                ));
                texture.set_data(utils::egui_image_bytes(&delta.image));

                let mut material = Material::new(EGUI_SHADER.clone(), map!["DITHERING" => 0u64]);

                material
                    .raw_bind_group_mut()
                    .set_parameter(
                        "ui_texture_sampler",
                        MaterialParameter::Sampler(sampler.clone()),
                        queue,
                    )
                    .unwrap();

                material
                    .raw_bind_group_mut()
                    .set_parameter(
                        "ui_texture",
                        MaterialParameter::Texture2D(texture.clone()),
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
                    *tex_id,
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

    /// Removes the provided textures from the map
    pub fn free_removed(&self, to_free: impl IntoIterator<Item = egui::TextureId>) {
        profiling::function_scope!();

        let mut texture_map = self.0.lock().unwrap();

        for item in to_free {
            texture_map.remove(&item);
        }
    }
}

/// A material for rendering one texture. All [`egui`] meshes use the same shader,
/// so we can just create a different material per texture to reduce bindgroup updates
#[derive(Debug)]
struct TextureMaterial {
    /// The texture this material is for
    texture: Arc<Texture>,

    /// The sampler used to sample [`Self::texture`]
    sampler: Arc<Sampler>,

    /// The actual material
    material: Material,

    /// The last screen size set on [`Self::material`]
    cur_screen_size: (f32, f32),
}

impl TextureMaterial {
    /// Sets the `screen_size` parameter on the given texture material, if the screen size
    /// has changed
    fn set_surface_size_if_changed(&mut self, sfc_size: (f32, f32), queue: &wgpu::Queue) {
        if self.cur_screen_size == sfc_size {
            return;
        }

        self.material
            .raw_bind_group_mut()
            .set_parameter(
                "screen_size",
                MaterialParameter::Vec2(vec2(sfc_size.0, sfc_size.1)),
                queue,
            )
            .unwrap();

        self.cur_screen_size = sfc_size;
    }
}
