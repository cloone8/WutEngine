//! UI functionality

use std::any::Any;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::builtins::assets::Texture;
use crate::builtins::components::ui::ScreenSpaceUICanvas;
use crate::component::Component;
use crate::component::data::ComponentData;
use crate::gameobject::GameObjectId;
use crate::plugins::{Context, WutEnginePlugin};
use crate::windowing::display::get_display;
use crate::windowing::window::{WindowData, WindowState};

pub use egui;
use egui::{PlatformOutput, RawInput, TexturesDelta, ViewportIdMap, ViewportOutput, emath};
use wutengine_core::identifiers::WindowIdentifier;
use wutengine_graphics::image::{ColorType, DynamicImage, RgbaImage};
use wutengine_graphics::texture::{TextureFiltering, TextureWrapping, WrappingMethod};

/// The main UI plugin for WutEngine
#[derive(Debug, Default)]
pub struct UIPlugin {
    screenspace_canvases: Mutex<Vec<GameObjectId>>,
    egui_context: egui::Context,
    texture_map: HashMap<egui::TextureId, Texture>,
    texture_gc_list: Vec<Texture>,
}

impl WutEnginePlugin for UIPlugin {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn pre_render(&mut self, context: &mut Context) {
        self.run_texture_gc();

        let screenspace_canvases = self.screenspace_canvases.lock().unwrap();

        for canvas_obj in screenspace_canvases.iter().copied() {
            let output =
                match get_screenspace_canvas_output(&mut self.egui_context, context, canvas_obj) {
                    Ok(o) => o,
                    Err(_e) => {
                        continue;
                    }
                };

            handle_platform_output(output.platform_output);

            handle_viewport_output(output.viewport_output);

            update_textures(
                output.textures_delta,
                &mut self.texture_map,
                &mut self.texture_gc_list,
            );

            let tesselated = self
                .egui_context
                .tessellate(output.shapes, output.pixels_per_point);
        }
    }
}

impl UIPlugin {
    /// Registers a new root screenspace canvas
    pub(crate) fn register_screenspace_canvas(&self, object: GameObjectId) {
        self.screenspace_canvases.lock().unwrap().push(object);
    }

    /// Deregisters a root screenspace canvas
    pub(crate) fn deregister_screenspace_canvas(&self, object: GameObjectId) {
        self.screenspace_canvases
            .lock()
            .unwrap()
            .retain(|cvs| *cvs != object);
    }

    fn run_texture_gc(&mut self) {
        //TODO: Actual GC
        self.texture_gc_list.clear();
    }
}

fn handle_platform_output(output: PlatformOutput) {}

fn handle_viewport_output(output: ViewportIdMap<ViewportOutput>) {}

fn update_textures(
    delta: TexturesDelta,
    texmap: &mut HashMap<egui::TextureId, Texture>,
    gc: &mut Vec<Texture>,
) {
    for to_free in delta.free {
        gc.push(match texmap.remove(&to_free) {
            Some(t) => t,
            None => {
                log::warn!(
                    "egui wants to remove texture {:?}, but that ID is not known by the UI plugin",
                    to_free
                );
                continue;
            }
        });
    }

    for (changed_tex, content) in delta.set {
        let tex = texmap.entry(changed_tex).or_default();

        if content.options.magnification != content.options.minification {
            log::warn!(
                "Different texture min/mag filters set. This is not supported, and mag will be used: min({:?}), mag({:?})",
                content.options.minification,
                content.options.magnification
            );
        }

        tex.set_filter(to_native_tex_filter(content.options.magnification));
        tex.set_wrapping(to_native_tex_wrapping(content.options.wrap_mode));

        match content.pos {
            Some([x, y]) => {
                // Update a sub-part of the image
            }
            None => {
                log::info!("Uploading image");
                // Update the entire image
                match content.image {
                    egui::ImageData::Color(color_image) => {
                        let img = RgbaImage::from_vec(
                            color_image.width() as u32,
                            color_image.height() as u32,
                            Vec::from(bytemuck::cast_slice::<_, u8>(color_image.pixels.as_slice())),
                        )
                        .unwrap();

                        tex.set_image(img);
                    }
                    egui::ImageData::Font(font_image) => {
                        let font_buf: Vec<u8> = font_image
                            .srgba_pixels(None)
                            .flat_map(|a| a.to_array())
                            .collect();

                        let img = RgbaImage::from_vec(
                            font_image.width() as u32,
                            font_image.height() as u32,
                            font_buf,
                        )
                        .unwrap();

                        tex.set_image(img);
                    }
                }
            }
        }
    }
}

fn get_screenspace_canvas_output(
    egui_context: &mut egui::Context,
    context: &mut Context,
    canvas_obj: GameObjectId,
) -> Result<egui::FullOutput, ()> {
    let gameobject = match context.gameobjects.get_object(canvas_obj) {
        Some(go) => go,
        None => {
            log::error!(
                "Could not find canvas GameObject for canvas with GameObject ID {}",
                canvas_obj
            );
            return Err(());
        }
    };

    let components = gameobject.components.borrow();
    let canvas_component = match find_component_of_type::<ScreenSpaceUICanvas>(&components) {
        Some(comp) => comp,
        None => {
            log::error!(
                "Could not find screen space UI canvas component for gameobject {}",
                canvas_obj
            );
            return Err(());
        }
    };

    let this_viewport = egui::ViewportId::from_hash_of(canvas_component.window.clone());
    let viewport_map = construct_viewport_map(context.windows.windows);

    if !viewport_map.contains_key(&this_viewport) {
        return Err(());
    }

    let input = RawInput {
        viewport_id: this_viewport,
        viewports: viewport_map,
        screen_rect: Default::default(),
        max_texture_side: Default::default(),
        time: Default::default(),
        predicted_dt: Default::default(),
        modifiers: Default::default(),
        events: Default::default(),
        hovered_files: Default::default(),
        dropped_files: Default::default(),
        focused: Default::default(),
        system_theme: Default::default(),
    };

    Ok(egui_context.run(input, |ctx| {
        canvas_component.run_ui(ctx);
    }))
}

fn find_component_of_type<T: Component>(components: &[ComponentData]) -> Option<&T> {
    for component in components.iter() {
        if let Some(cast) = component.get_inner_cast::<T>() {
            return Some(cast);
        }
    }

    None
}

fn construct_viewport_map(
    windows: &HashMap<WindowIdentifier, WindowData>,
) -> ViewportIdMap<egui::ViewportInfo> {
    let mut m = ViewportIdMap::default();

    for (ident, data) in windows {
        m.insert(
            egui::ViewportId::from_hash_of(ident),
            to_viewport_info(data),
        );
    }

    m
}

fn to_viewport_info(windowdata: &WindowData) -> egui::ViewportInfo {
    let display = windowdata.current_monitor.as_ref().and_then(get_display);

    egui::ViewportInfo {
        parent: None,
        title: Some(windowdata.title.clone()),
        events: Vec::new(), //todo
        native_pixels_per_point: None,
        monitor_size: display.map(|disp| {
            let size = disp.handle.size();
            emath::vec2(size.width as f32, size.height as f32)
        }),
        inner_rect: Some(egui::Rect::from_min_size(
            emath::Pos2::ZERO,
            emath::Vec2::new(windowdata.size.0 as f32, windowdata.size.1 as f32),
        )),
        outer_rect: Some(egui::Rect::from_min_size(
            emath::Pos2::ZERO,
            emath::Vec2::new(
                windowdata.outer_size.0 as f32,
                windowdata.outer_size.1 as f32,
            ),
        )),
        minimized: Some(windowdata.state == WindowState::Minimized),
        maximized: Some(windowdata.state == WindowState::Maximized),
        fullscreen: Some(windowdata.is_fullscreen),
        focused: Some(true), //todo
    }
}

const fn to_native_tex_filter(filter: egui::TextureFilter) -> TextureFiltering {
    match filter {
        egui::TextureFilter::Nearest => TextureFiltering::Nearest,
        egui::TextureFilter::Linear => TextureFiltering::Linear,
    }
}

const fn to_native_tex_wrapping(wrapping: egui::TextureWrapMode) -> TextureWrapping {
    match wrapping {
        egui::TextureWrapMode::ClampToEdge => TextureWrapping::Both(WrappingMethod::Clamp),
        egui::TextureWrapMode::Repeat => TextureWrapping::Both(WrappingMethod::Repeat),
        egui::TextureWrapMode::MirroredRepeat => TextureWrapping::Both(WrappingMethod::Mirror),
    }
}
