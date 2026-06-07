//! Module housing the window manager

use alloc::sync::Arc;
use std::sync::RwLock;
use wutengine_graphics::wgpu;

use nohash_hasher::IntMap;
use smallvec::SmallVec;

use crate::config;
use crate::graphics;
use wutengine_util::{InitOnce, assert_main_thread};

use super::Display;
use super::Window;

/// The global [WindowManager]
static WINDOW_MANAGER: InitOnce<RwLock<WindowManager>> = InitOnce::new();

/// Initializes the global window management subsystem
pub(crate) fn init() {
    InitOnce::init(&WINDOW_MANAGER, RwLock::new(WindowManager::new()));

    #[cfg(feature = "development_overlay")]
    {
        crate::development_overlay::add_development_overlay_window(
            development_overlay::WindowManagerOverlay::default(),
        );
    }
}

/// Registers a newly created window
///
/// Must be called from the main thread
pub(crate) fn new_window(
    id: crate::window::Window,
    window: Arc<winit::window::Window>,
    surface: wgpu::Surface<'static>,
) {
    profiling::function_scope!();

    assert_main_thread!();

    let native_id = window.id();
    let info = WindowInfo::new(id, window, surface);

    let mut window_manager = WINDOW_MANAGER.write().unwrap();
    window_manager.winit_to_engine.insert(native_id.into(), id);
    window_manager.windows.insert(id, info);
}

/// Returns the size of the given window in pixels.
///
/// If the window does not exists, returns [None]
pub(crate) fn get_size(id: Window) -> Option<(u32, u32)> {
    profiling::function_scope!();

    let window_manager = WINDOW_MANAGER.read().unwrap();
    window_manager.windows.get(&id).map(|info| info.inner_size)
}

/// Returns the scale factor for the given window
///
/// If the window does not exists, returns [None]
pub(crate) fn get_scale_factor(id: Window) -> Option<f64> {
    profiling::function_scope!();

    let window_manager = WINDOW_MANAGER.read().unwrap();
    window_manager
        .windows
        .get(&id)
        .map(|info| info.os_scale_factor)
}

/// Instructs the [WindowManager] to destroy the given window.
///
/// If the window does not exist, does nothing.
///
/// Must be called from the main thread
pub(crate) fn close_window(id: crate::window::Window) {
    profiling::function_scope!();

    assert_main_thread!();

    let mut window_manager = WINDOW_MANAGER.write().unwrap();

    let Some(info) = window_manager.windows.remove(&id) else {
        log::warn!("Window {id} was already closed");
        return;
    };

    let removed = window_manager
        .winit_to_engine
        .remove(&info.native.id().into());
    assert!(removed.is_some(), "Native to engine ID map was missing");
}

/// Updates the icon of the given window to the new one.
///
/// If the window does not exist, does nothing.
///
/// Must be called from the main thread
pub(crate) fn set_icon(id: crate::window::Window, icon: winit::window::Icon) {
    profiling::function_scope!();

    assert_main_thread!();

    let window_manager = WINDOW_MANAGER.read().unwrap();

    let Some(window) = window_manager.windows.get(&id) else {
        log::error!("Could not update icon for window {id} because it could not be found");
        return;
    };

    #[cfg(windows)]
    {
        use winit::platform::windows::WindowExtWindows;

        window.native.set_taskbar_icon(Some(icon.clone()));
    }

    window.native.set_window_icon(Some(icon));
}

/// Refreshes the cached info for a given [WindowId]. Should be called by the main
/// engine runtime when a relevant [winit] event is received, or if some other window
/// attribute was changed.
///
/// Must be called from the main thread
pub(crate) fn refresh_cached_info(id: &crate::window::Window) {
    profiling::function_scope!();

    assert_main_thread!();

    log::trace!("Refreshing cached window info for {id}");

    let mut window_manager = WINDOW_MANAGER.write().unwrap();

    let Some(window) = window_manager.windows.get_mut(id) else {
        log::error!("Could not refresh cached info for window {id} because it could not be found");
        return;
    };

    window.refresh();
    window.reconfigure_surface();
}

/// Refreshes the known displays
pub(crate) fn refresh_display_info(event_loop: &winit::event_loop::ActiveEventLoop) {
    profiling::function_scope!();
    assert_main_thread!();

    let primary_display = event_loop.primary_monitor();
    let all_displays = event_loop.available_monitors().collect::<Vec<_>>();

    let mut window_manager = WINDOW_MANAGER.write().unwrap();
    window_manager.primary_display = None;

    let mut new_display_map = IntMap::default();

    for monitor_handle in all_displays {
        let id = find_existing_display_id(&window_manager, &monitor_handle).unwrap_or_default();

        let mut is_primary = false;
        if let Some(primary_display) = &primary_display
            && &monitor_handle == primary_display
        {
            window_manager.primary_display = Some(id);
            is_primary = true;
        }

        let info = DisplayInfo::from_monitor_handle(id, monitor_handle, is_primary);

        new_display_map.insert(id, info);
    }

    window_manager.displays = new_display_map;
}

fn find_existing_display_id(
    window_manager: &WindowManager,
    monitor_handle: &winit::monitor::MonitorHandle,
) -> Option<Display> {
    for (id, info) in window_manager.displays.iter() {
        if &info.handle == monitor_handle {
            return Some(*id);
        }
    }

    None
}

/// For a given [winit::window::WindowId], returns the WutEngine [WindowId] if one can be found
pub(crate) fn find_id(native_id: winit::window::WindowId) -> Option<Window> {
    profiling::function_scope!();

    let window_manager = WINDOW_MANAGER.read().unwrap();

    window_manager
        .winit_to_engine
        .get(&native_id.into())
        .copied()
}

/// Returns the surface textures for all current windows that have a valid one.
/// If vsync is enabled, this is the point where the CPU is locked
pub(crate) fn get_surface_textures() -> SmallVec<[(Window, wgpu::SurfaceTexture); 2]> {
    profiling::function_scope!();

    let window_manager = WINDOW_MANAGER.read().unwrap();

    let mut surfaces: SmallVec<[_; 2]> = SmallVec::new_const();

    for (&id, window_info) in window_manager.windows.iter() {
        log::trace!("Getting surface texture for window {id}");

        if let Some(surface_tex) = unwrap_surface_tex(&window_info.surface, id) {
            surfaces.push((id, surface_tex));
        }
    }

    surfaces
}

/// Notifies the given windows that content is about to be presented to them
pub(crate) fn pre_present_notify<'a>(windows: impl IntoIterator<Item = &'a Window>) {
    profiling::function_scope!();

    let window_manager = WINDOW_MANAGER.read().unwrap();

    for window in windows {
        if let Some(window_info) = window_manager.windows.get(window) {
            window_info.native.pre_present_notify();
        } else {
            log::warn!(
                "Could not pre-present notify window {window} because it could not be found"
            );
        }
    }
}

pub(crate) fn get_displays() -> Vec<Display> {
    WINDOW_MANAGER
        .read()
        .unwrap()
        .displays
        .keys()
        .copied()
        .collect()
}

pub(crate) fn primary_display() -> Option<Display> {
    let window_manager = WINDOW_MANAGER.read().unwrap();

    if let Some(prim_id) = window_manager.primary_display {
        return Some(prim_id);
    }

    // Fallback: if we can't determine the primary display just return an arbitrary one
    window_manager.displays.keys().copied().next()
}

pub(crate) fn monitor_handle_from_display(id: Display) -> Option<winit::monitor::MonitorHandle> {
    let window_manager = WINDOW_MANAGER.read().unwrap();

    window_manager
        .displays
        .get(&id)
        .map(|info| info.handle.clone())
}

pub(crate) fn exclusive_fullscreen_modes(
    id: Display,
) -> Option<Vec<DisplayExclusiveFullscreenMode>> {
    let window_manager = WINDOW_MANAGER.read().unwrap();

    window_manager
        .displays
        .get(&id)
        .map(|info| info.video_modes.clone())
}

fn unwrap_surface_tex(surface: &wgpu::Surface, window: Window) -> Option<wgpu::SurfaceTexture> {
    match surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Success(sfctex) => Some(sfctex),
        wgpu::CurrentSurfaceTexture::Suboptimal(sfctex) => {
            log::warn!("Suboptimal surface for window {window}, should recreate");
            Some(sfctex)
        }
        wgpu::CurrentSurfaceTexture::Timeout => {
            panic!("Timeout while trying to obtain surface texture for window {window}");
        }
        wgpu::CurrentSurfaceTexture::Occluded => {
            log::trace!("Surface texture for window {window} is occluded");
            None
        }
        wgpu::CurrentSurfaceTexture::Outdated => {
            log::error!("Surface texture for window {window} is outdated");
            None
        }
        wgpu::CurrentSurfaceTexture::Lost => {
            panic!("Surface texture for window {window} lost");
        }
        wgpu::CurrentSurfaceTexture::Validation => {
            panic!(
                "Validation error while trying to obtain the surface texture for window {window}"
            );
        }
    }
}

/// The WutEngine window manager. Owns all native windows, and allows interaction with them
#[derive(Debug)]
struct WindowManager {
    /// Mapping from [winit] window IDs to WutEngine window IDs.
    /// use the [Into::into] implementation on [winit::window::WindowId] to convert
    /// it to a [u64] for use in this map
    winit_to_engine: IntMap<u64, crate::window::Window>,

    windows: IntMap<crate::window::Window, WindowInfo>,

    primary_display: Option<crate::window::Display>,
    displays: IntMap<crate::window::Display, DisplayInfo>,
}

#[derive(Debug)]
struct WindowInfo {
    /// The engine-internal ID
    id: Window,

    /// The actual native window handle
    native: Arc<winit::window::Window>,

    surface: wgpu::Surface<'static>,

    /// The physical window size, in pixels `W x H`
    inner_size: (u32, u32),

    /// The OS-provided scale factor for the window
    os_scale_factor: f64,
}

impl WindowInfo {
    /// Creates a new [WindowInfo] struct, containing cached
    /// window information to prevent problems with querying information
    /// on non-main threads
    pub(crate) fn new(
        id: Window,
        native: Arc<winit::window::Window>,
        surface: wgpu::Surface<'static>,
    ) -> Self {
        let mut new = Self {
            id,
            native,
            surface,
            inner_size: (0, 0),
            os_scale_factor: 1.0,
        };

        new.refresh();
        new.reconfigure_surface();

        new
    }

    /// Refresh all cached window information
    fn refresh(&mut self) {
        assert_main_thread!();

        log::trace!("Refreshing cached information for window {}", self.id);

        self.inner_size = self.native.inner_size().into();
        self.os_scale_factor = self.native.scale_factor();
    }

    fn reconfigure_surface(&self) {
        log::debug!("Reconfiguring surface for window {}", self.id);

        let size = self.inner_size;
        let surface = &self.surface;
        let surface_caps = surface.get_capabilities(graphics::adapter());

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let present_mode = get_best_present_mode(
            self.id,
            config::try_get("wutengine.window.vsync").unwrap_or(true),
            &surface_caps.present_modes,
        );

        log::debug!("Chose present mode {present_mode:?} for window {}", self.id);

        let desired_maximum_frame_latency =
            if config::try_get("wutengine.window.triple_buffering").unwrap_or(false) {
                2
            } else {
                1
            };

        log::debug!("Requested maximum frame latency: {desired_maximum_frame_latency}");

        surface.configure(
            graphics::device(),
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.0,
                height: size.1,
                present_mode,
                desired_maximum_frame_latency,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![
                    surface_format.remove_srgb_suffix(),
                    surface_format.add_srgb_suffix(),
                ],
            },
        );
    }
}

fn get_best_present_mode(
    window: Window,
    wants_vsync: bool,
    capabilities: &[wgpu::PresentMode],
) -> wgpu::PresentMode {
    log::trace!(
        "Window {} supports present modes: {capabilities:?}. Vsync requested: {wants_vsync}",
        window
    );

    if wants_vsync {
        if capabilities.contains(&wgpu::PresentMode::FifoRelaxed) {
            wgpu::PresentMode::FifoRelaxed
        } else {
            wgpu::PresentMode::Fifo
        }
    } else {
        if capabilities.contains(&wgpu::PresentMode::Immediate) {
            wgpu::PresentMode::Immediate
        } else if capabilities.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else {
            wgpu::PresentMode::Fifo
        }
    }
}

impl WindowManager {
    /// Creates a new default [WindowManager]
    pub(crate) fn new() -> Self {
        Self {
            winit_to_engine: IntMap::default(),
            windows: IntMap::default(),
            primary_display: None,
            displays: IntMap::default(),
        }
    }
}

#[derive(Debug)]
struct DisplayInfo {
    handle: winit::monitor::MonitorHandle,
    name: Option<String>,
    size: (u32, u32),
    scaling_factor: f64,
    video_modes: Vec<DisplayExclusiveFullscreenMode>,
    is_primary: bool,
}

#[derive(Debug, Clone)]
pub struct DisplayExclusiveFullscreenMode(
    pub(super) Display,
    pub(super) winit::monitor::VideoModeHandle,
);

impl DisplayInfo {
    fn from_monitor_handle(
        id: Display,
        handle: winit::monitor::MonitorHandle,
        is_primary: bool,
    ) -> Self {
        profiling::function_scope!();
        assert_main_thread!();

        Self {
            name: handle.name(),
            size: (handle.size().width, handle.size().height),
            scaling_factor: handle.scale_factor(),
            video_modes: handle
                .video_modes()
                .map(|videomode| DisplayExclusiveFullscreenMode(id, videomode))
                .collect(),
            is_primary,
            handle,
        }
    }
}

#[cfg(feature = "development_overlay")]
mod development_overlay {

    use wutengine_development_overlay::wutengine_egui::egui;
    use wutengine_graphics::wgpu;

    use crate::development_overlay::DevelopmentOverlayWindow;

    use super::WINDOW_MANAGER;

    #[derive(Default)]
    pub(super) struct WindowManagerOverlay {}

    impl DevelopmentOverlayWindow for WindowManagerOverlay {
        fn name(&self) -> &str {
            "Windows"
        }

        fn icon(&self) -> Option<&str> {
            Some("🪟")
        }

        fn show(&mut self, ui: &mut egui::Ui) {
            let win_man = WINDOW_MANAGER.read().unwrap();
            let mut windows = Vec::new();
            for (id, info) in win_man.windows.iter() {
                windows.push(*id);

                egui::CollapsingHeader::new(id.to_string())
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label(format!("Size: {}x{}", info.inner_size.0, info.inner_size.1));
                        ui.label(format!("OS scale factor: {}", info.os_scale_factor));

                        let Some(surface_config) = info.surface.get_configuration() else {
                            return;
                        };

                        let wgpu::SurfaceConfiguration {
                            usage,
                            format,
                            width: _,
                            height: _,
                            present_mode,
                            desired_maximum_frame_latency,
                            alpha_mode,
                            view_formats,
                        } = surface_config;

                        ui.label(format!("Format: {format:?}"));

                        ui.label(format!("View formats:"));
                        ui.indent(id, |ui| {
                            for tex_format in view_formats {
                                ui.label(format!("{tex_format:?}"));
                            }
                        });

                        ui.label(format!("Usages:"));
                        ui.indent(id, |ui| {
                            for (usage, _) in usage.iter_names() {
                                ui.label(usage);
                            }
                        });

                        ui.label(format!("Present mode: {present_mode:?}"));
                        ui.label(format!(
                            "Desired frame latency: {desired_maximum_frame_latency}"
                        ));

                        ui.label(format!("Alpha mode: {alpha_mode:?}"));
                    });
            }

            drop(win_man);

            ui.separator();

            if ui.button("Reconfigure all").clicked() {
                for window in windows {
                    crate::runtime::notify_event_loop(
                        crate::runtime::WinitEvent::ForceSurfaceReconfigure(window),
                    );
                }
            }
        }
    }
}
