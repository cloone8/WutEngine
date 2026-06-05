//! Development overlay tools and API

use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use glam::Vec2;
use glam::Vec3;
use glam::Vec4;
use rendering_test::ColorTest;
use wutengine_asset::AssetHandle;
use wutengine_asset::assets::mesh::MeshIndices;
use wutengine_asset::assets::mesh::MeshTopology;
use wutengine_asset::assets::mesh::SerializedMesh;
use wutengine_asset::assets::sampler::FilterMode;
use wutengine_asset::assets::sampler::WrapMode;
use wutengine_asset::assets::sampler::WrapModeType;
use wutengine_asset::assets::texture::TextureConfig;
use wutengine_asset::assets::texture::TextureFormat;
use wutengine_shadercompiler::MATERIAL_PARAMS_BIND_GROUP_INDEX;

use crate::graphics;
use crate::graphics::material::Material;
use crate::graphics::material::MaterialParameter;
use crate::graphics::mesh::Mesh;
use crate::graphics::sampler::Sampler;
use crate::graphics::texture::Texture;
use crate::util::InitOnce;
use crate::util::map;

mod rendering_test;

pub(crate) static DEV_OVERLAY: InitOnce<DevOverlayManager> = InitOnce::new();

pub(crate) fn init() {
    InitOnce::init(&DEV_OVERLAY, DevOverlayManager::new());
}

pub(crate) struct DevOverlayManager {
    active: AtomicBool,
    egui_context: egui::Context,
    textures: Mutex<HashMap<egui::TextureId, (Texture, Sampler)>>,
    gallery: Mutex<WidgetGallery>,
    rendering_test: Mutex<ColorTest>,
}

impl DevOverlayManager {
    fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            egui_context: egui::Context::default(),
            textures: Mutex::new(HashMap::new()),
            gallery: Mutex::new(WidgetGallery::default()),
            rendering_test: Mutex::new(ColorTest::default()),
        }
    }
}

pub(crate) fn render_if_active(surface: &wgpu::SurfaceTexture) -> Option<wgpu::CommandBuffer> {
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
    let sfc_rect = egui::Rect {
        min: egui::Pos2::ZERO,
        max: egui::Pos2::new(sfc_size.0 as f32, sfc_size.1 as f32),
    };

    let egui_input = egui::RawInput {
        viewport_id: egui::ViewportId::ROOT,
        viewports: map![
            egui::ViewportId::ROOT => egui::ViewportInfo {
                parent: None,
                title: Some("Development Overlay".to_string()),
                events: vec![],
                native_pixels_per_point: None,
                monitor_size: None,
                inner_rect: Some(sfc_rect),
                outer_rect: None,
                minimized: None,
                maximized: None,
                fullscreen: None,
                focused: Some(true),
                occluded: None
            }
        ],
        safe_area_insets: None,
        screen_rect: Some(sfc_rect),
        max_texture_side: None,
        time: Some(crate::time::unscaled_time64()),
        predicted_dt: 1.0 / 60.0,
        modifiers: egui::Modifiers::NONE,
        events: vec![],
        hovered_files: vec![],
        dropped_files: vec![],
        focused: true,
        system_theme: None,
    };

    let mut gallery = DEV_OVERLAY.gallery.lock().unwrap();
    let mut rendering_test = DEV_OVERLAY.rendering_test.lock().unwrap();
    let egui_output = DEV_OVERLAY.egui_context.run_ui(egui_input, |ui| {
        ui.label("WutEngine Development Overlay");

        let mut gallery_open = true;
        gallery.show(ui, &mut gallery_open);

        let mut rendering_test_open = true;
        egui::Window::new("Rendering Test")
            .open(&mut rendering_test_open)
            .default_pos((512.0, 0.0))
            .resizable([true, false]) // resizable so we can shrink if the text edit grows
            .default_width(480.0)
            .constrain_to(ui.available_rect_before_wrap())
            .show(ui, |ui| {
                rendering_test.ui(ui);
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

    upload_new_textures(&mut texture_map, egui_output.textures_delta.set);

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

    for primitive in clipped_output {
        render_pass.push_debug_group("Render single primitive");

        render_primitive(
            primitive,
            &mut render_pass,
            &texture_map,
            &mut cur_pipeline,
            surface_format,
            sfc_size,
            egui_output.pixels_per_point,
        );

        render_pass.pop_debug_group();
    }

    drop(render_pass);
    command_encoder.pop_debug_group();

    free_removed_textures(&mut texture_map, egui_output.textures_delta.free);

    Some(command_encoder.finish())
}

fn render_primitive(
    primitive: egui::ClippedPrimitive,
    pass: &mut wgpu::RenderPass,
    texture_map: &HashMap<egui::TextureId, (Texture, Sampler)>,
    current_pipeline: &mut Option<Arc<wgpu::RenderPipeline>>,
    surface_format: wgpu::TextureFormat,
    surface_size: (u32, u32),
    pixels_per_point: f32,
) {
    profiling::function_scope!();

    let mut material = Material::new(crate::builtins::shaders::EGUI.get_arc().unwrap(), map![]);

    let pipeline = graphics::pipeline::get_pipeline(
        &material,
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

    let scissor_rect = ScissorRect::new(&primitive.clip_rect, pixels_per_point, surface_size);
    pass.set_scissor_rect(
        scissor_rect.x,
        scissor_rect.y,
        scissor_rect.width,
        scissor_rect.height,
    );

    material
        .user_bind_group
        .set_parameter(
            "screen_size",
            MaterialParameter::Vec2(Vec2::new(surface_size.0 as f32, surface_size.1 as f32)),
            crate::graphics::queue(),
        )
        .unwrap();

    match primitive.primitive {
        egui::epaint::Primitive::Mesh(egui_mesh) => {
            let mut vertices = Vec::with_capacity(egui_mesh.vertices.len());
            let mut uvs = Vec::with_capacity(egui_mesh.vertices.len());
            let mut colors = Vec::with_capacity(egui_mesh.vertices.len());

            for vertex in egui_mesh.vertices {
                vertices.push(Vec3::new(vertex.pos.x, vertex.pos.y, 0.0));
                uvs.push(Vec2::new(vertex.uv.x, vertex.uv.y));
                colors.push(Vec4::new(
                    vertex.color.r() as f32 / 255.0,
                    vertex.color.g() as f32 / 255.0,
                    vertex.color.b() as f32 / 255.0,
                    vertex.color.a() as f32 / 255.0,
                ));
            }

            let mesh_data = SerializedMesh {
                vertices,
                indices: MeshIndices::U32(egui_mesh.indices),
                uvs: map![
                    0 => uvs
                ],
                colors,
                topology: MeshTopology::Triangle,
                keep_data: false,
            };

            let Some(mesh) = Mesh::new(&mesh_data) else {
                return;
            };

            let texture = texture_map.get(&egui_mesh.texture_id).unwrap();

            material
                .user_bind_group
                .set_parameter(
                    "ui_texture",
                    MaterialParameter::Texture2D(AssetHandle::new(texture.0.clone())),
                    crate::graphics::queue(),
                )
                .unwrap();

            material
                .user_bind_group
                .set_parameter(
                    "ui_texture_sampler",
                    MaterialParameter::Sampler(AssetHandle::new(texture.1.clone())),
                    crate::graphics::queue(),
                )
                .unwrap();

            material
                .user_bind_group
                .update_bind_group(crate::graphics::device());

            pass.set_bind_group(
                MATERIAL_PARAMS_BIND_GROUP_INDEX,
                material.user_bind_group.get_bind_group().unwrap(),
                &[],
            );

            let attrs = &material.compiled_shader.vertex_attributes;

            for (attr_type, attr_info) in attrs {
                let Some(vertex_buffer) = mesh.vertex_buffers.get(attr_type) else {
                    log::error!(
                        "Mesh is missing vertex buffer for requested attribute: {attr_type}"
                    );
                    return;
                };

                pass.set_vertex_buffer(attr_info.shader_location, vertex_buffer.buffer.slice(..));
            }

            pass.set_index_buffer(
                mesh.index_buffer.buffer.slice(..),
                mesh.index_buffer.format.to_wgpu(),
            );

            pass.draw_indexed(0..mesh.index_buffer.count as u32, 0, 0..1);
        }
        egui::epaint::Primitive::Callback(_) => unreachable!("Not supported"),
    }
}

fn upload_new_textures(
    texture_map: &mut HashMap<egui::TextureId, (Texture, Sampler)>,
    to_set: Vec<(egui::TextureId, egui::epaint::ImageDelta)>,
) {
    profiling::function_scope!();

    for (tex_id, delta) in to_set {
        let sampler = sampler_from_egui(&delta.options);

        match delta.pos {
            Some(pos) => {
                let (texture, cur_sampler) = texture_map.get_mut(&tex_id).unwrap();

                *cur_sampler = sampler;

                texture.set_partial_data(
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

                texture_map.insert(tex_id, (texture, sampler));
            }
        }
    }
}

fn tex_config_from_egui_data(delta: &egui::epaint::ImageData) -> TextureConfig {
    match delta {
        egui::ImageData::Color(_) => TextureConfig {
            width: delta.width() as u32,
            height: delta.height() as u32,
            format: TextureFormat::Rgba8,
        },
    }
}

fn egui_image_bytes(image: &egui::epaint::ImageData) -> &[u8] {
    match image {
        egui::ImageData::Color(color_image) => bytemuck::cast_slice(&color_image.pixels),
    }
}

fn sampler_from_egui(options: &egui::TextureOptions) -> Sampler {
    let filtering = filter_mode_from_egui(options.magnification);
    let wrapping = wrap_mode_from_egui(options.wrap_mode);

    Sampler::new(filtering, wrapping)
}

fn filter_mode_from_egui(egui_mode: egui::TextureFilter) -> FilterMode {
    match egui_mode {
        egui::TextureFilter::Nearest => FilterMode::Nearest,
        egui::TextureFilter::Linear => FilterMode::Linear,
    }
}

fn wrap_mode_from_egui(egui_mode: egui::TextureWrapMode) -> WrapModeType {
    match egui_mode {
        egui::TextureWrapMode::ClampToEdge => WrapModeType::Single(WrapMode::Clamp),
        egui::TextureWrapMode::Repeat => WrapModeType::Single(WrapMode::Repeat),
        egui::TextureWrapMode::MirroredRepeat => WrapModeType::Single(WrapMode::MirrorRepeat),
    }
}

fn free_removed_textures(
    texture_map: &mut HashMap<egui::TextureId, (Texture, Sampler)>,
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

/// A Rect in physical pixel space, used for setting clipping rectangles.
struct ScissorRect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl ScissorRect {
    fn new(clip_rect: &egui::epaint::Rect, pixels_per_point: f32, target_size: (u32, u32)) -> Self {
        // Transform clip rect to physical pixels:
        let clip_min_x = pixels_per_point * clip_rect.min.x;
        let clip_min_y = pixels_per_point * clip_rect.min.y;
        let clip_max_x = pixels_per_point * clip_rect.max.x;
        let clip_max_y = pixels_per_point * clip_rect.max.y;

        // Round to integer:
        let clip_min_x = clip_min_x.round() as u32;
        let clip_min_y = clip_min_y.round() as u32;
        let clip_max_x = clip_max_x.round() as u32;
        let clip_max_y = clip_max_y.round() as u32;

        // Clamp:
        let clip_min_x = clip_min_x.clamp(0, target_size.0);
        let clip_min_y = clip_min_y.clamp(0, target_size.1);
        let clip_max_x = clip_max_x.clamp(clip_min_x, target_size.0);
        let clip_max_y = clip_max_y.clamp(clip_min_y, target_size.0);

        Self {
            x: clip_min_x,
            y: clip_min_y,
            width: clip_max_x - clip_min_x,
            height: clip_max_y - clip_min_y,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Enum {
    First,
    Second,
    Third,
}

/// Shows off one example of each major type of widget.
pub struct WidgetGallery {
    enabled: bool,
    visible: bool,
    boolean: bool,
    opacity: f32,
    radio: Enum,
    scalar: f32,
    string: String,
    color: egui::Color32,
    animate_progress_bar: bool,
}

impl Default for WidgetGallery {
    fn default() -> Self {
        Self {
            enabled: true,
            visible: true,
            opacity: 1.0,
            boolean: false,
            radio: Enum::First,
            scalar: 42.0,
            string: Default::default(),
            color: egui::Color32::LIGHT_BLUE.linear_multiply(0.5),
            animate_progress_bar: false,
        }
    }
}

impl WidgetGallery {
    fn name(&self) -> &'static str {
        "🗄 Widget Gallery"
    }

    fn show(&mut self, ui: &mut egui::Ui, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .resizable([true, false]) // resizable so we can shrink if the text edit grows
            .default_width(280.0)
            .constrain_to(ui.available_rect_before_wrap())
            .show(ui, |ui| {
                self.ui(ui);
            });
    }
}

impl WidgetGallery {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let mut ui_builder = egui::UiBuilder::new();
        if !self.enabled {
            ui_builder = ui_builder.disabled();
        }
        if !self.visible {
            ui_builder = ui_builder.invisible();
        }

        ui.scope_builder(ui_builder, |ui| {
            ui.multiply_opacity(self.opacity);

            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    self.gallery_grid_contents(ui);
                });
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.visible, "Visible")
                .on_hover_text("Uncheck to hide all the widgets.");
            if self.visible {
                ui.checkbox(&mut self.enabled, "Interactive")
                    .on_hover_text("Uncheck to inspect how the widgets look when disabled.");
                (ui.add(
                    egui::DragValue::new(&mut self.opacity)
                        .speed(0.01)
                        .range(0.0..=1.0),
                ) | ui.label("Opacity"))
                .on_hover_text("Reduce this value to make widgets semi-transparent");
            }
        });

        ui.separator();

        ui.vertical_centered(|ui| {
            let tooltip_text = "The full egui documentation.\nYou can also click the different widgets names in the left column.";
            ui.hyperlink("https://docs.rs/egui/").on_hover_text(tooltip_text);
        });
    }
}

impl WidgetGallery {
    fn gallery_grid_contents(&mut self, ui: &mut egui::Ui) {
        let Self {
            enabled: _,
            visible: _,
            opacity: _,
            boolean,
            radio,
            scalar,
            string,
            color,
            animate_progress_bar,
        } = self;

        ui.add(doc_link_label("Label", "label"));
        ui.label("Welcome to the widget gallery!");
        ui.end_row();

        ui.add(doc_link_label("Hyperlink", "Hyperlink"));
        use egui::special_emojis::GITHUB;
        ui.hyperlink_to(
            format!("{GITHUB} egui on GitHub"),
            "https://github.com/emilk/egui",
        );
        ui.end_row();

        ui.add(doc_link_label("TextEdit", "TextEdit"));
        ui.add(egui::TextEdit::singleline(string).hint_text("Write something here"));
        ui.end_row();

        ui.add(doc_link_label("Button", "button"));
        if ui.button("Click me!").clicked() {
            *boolean = !*boolean;
        }
        ui.end_row();

        ui.add(doc_link_label("Link", "link"));
        if ui.link("Click me!").clicked() {
            *boolean = !*boolean;
        }
        ui.end_row();

        ui.add(doc_link_label("Checkbox", "checkbox"));
        ui.checkbox(boolean, "Checkbox");
        ui.end_row();

        ui.add(doc_link_label("RadioButton", "radio"));
        ui.horizontal(|ui| {
            ui.radio_value(radio, Enum::First, "First");
            ui.radio_value(radio, Enum::Second, "Second");
            ui.radio_value(radio, Enum::Third, "Third");
        });
        ui.end_row();

        ui.add(doc_link_label("SelectableLabel", "SelectableLabel"));
        ui.horizontal(|ui| {
            ui.selectable_value(radio, Enum::First, "First");
            ui.selectable_value(radio, Enum::Second, "Second");
            ui.selectable_value(radio, Enum::Third, "Third");
        });
        ui.end_row();

        ui.add(doc_link_label("ComboBox", "ComboBox"));

        egui::ComboBox::from_label("Take your pick")
            .selected_text(format!("{radio:?}"))
            .show_ui(ui, |ui| {
                ui.selectable_value(radio, Enum::First, "First");
                ui.selectable_value(radio, Enum::Second, "Second");
                ui.selectable_value(radio, Enum::Third, "Third");
            });
        ui.end_row();

        ui.add(doc_link_label("Slider", "Slider"));
        ui.add(egui::Slider::new(scalar, 0.0..=360.0).suffix("°"));
        ui.end_row();

        ui.add(doc_link_label("DragValue", "DragValue"));
        ui.add(egui::DragValue::new(scalar).speed(1.0));
        ui.end_row();

        ui.add(doc_link_label("ProgressBar", "ProgressBar"));
        let progress = *scalar / 360.0;
        let progress_bar = egui::ProgressBar::new(progress)
            .show_percentage()
            .animate(*animate_progress_bar);
        *animate_progress_bar = ui
            .add(progress_bar)
            .on_hover_text("The progress bar can be animated!")
            .hovered();
        ui.end_row();

        ui.add(doc_link_label("Color picker", "color_edit"));
        ui.color_edit_button_srgba(color);
        ui.end_row();

        ui.add(doc_link_label("Image", "Image"));
        ui.end_row();

        ui.add(doc_link_label(
            "Button with image",
            "Button::image_and_text",
        ));
        ui.end_row();

        ui.add(doc_link_label("Separator", "separator"));
        ui.separator();
        ui.end_row();

        ui.add(doc_link_label("CollapsingHeader", "collapsing"));
        ui.collapsing("Click to see what is hidden!", |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("It's a ");
                ui.add(doc_link_label("Spinner", "spinner"));
                ui.add_space(4.0);
                ui.add(egui::Spinner::new());
            });
        });
        ui.end_row();

        ui.end_row();
    }
}

fn doc_link_label<'a>(title: &'a str, search_term: &'a str) -> impl egui::Widget + 'a {
    doc_link_label_with_crate("egui", title, search_term)
}

fn doc_link_label_with_crate<'a>(
    crate_name: &'a str,
    title: &'a str,
    search_term: &'a str,
) -> impl egui::Widget + 'a {
    let url = format!("https://docs.rs/{crate_name}?search={search_term}");
    move |ui: &mut egui::Ui| {
        ui.hyperlink_to(title, url).on_hover_ui(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Search egui docs for");
                ui.code(search_term);
            });
        })
    }
}
