//! Module housing the window manager

use alloc::sync::Arc;
use nohash_hasher::IntSet;
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

    let mut window_manager = WINDOW_MANAGER.write().unwrap();

    let native_id = window.id();
    let is_primary = window_manager.windows.is_empty(); // First window is always the primary window

    let info = WindowInfo::new(id, window.title(), is_primary, window, surface);

    window_manager.winit_to_engine.insert(native_id.into(), id);
    window_manager.windows.insert(id, info);
}

/// Locks the window manager, and calls `map_func` with an iterator containing all open windows
///
/// The window manager is locked for the duration of the callback
#[inline(always)]
pub(super) fn get_windows_and<T>(map_func: impl FnOnce(&IntMap<Window, WindowInfo>) -> T) -> T {
    profiling::function_scope!();

    let window_manager = WINDOW_MANAGER.read().unwrap();

    map_func(&window_manager.windows)
}

/// Locks the window manager, and calls `map_func` with a reference to the window info for `id`, if it exists.
///
/// The window manager is locked for the duration of the callback
#[inline(always)]
pub(super) fn get_window_and<T>(id: Window, map_func: impl FnOnce(Option<&WindowInfo>) -> T) -> T {
    profiling::function_scope!();

    let window_manager = WINDOW_MANAGER.read().unwrap();

    map_func(window_manager.windows.get(&id))
}

/// Instructs the [WindowManager] to destroy the given window.
///
/// If the window does not exist, does nothing.
///
/// Returns the amount of remaining windows.
///
/// Must be called from the main thread
pub(crate) fn close_window(id: crate::window::Window) -> usize {
    profiling::function_scope!();

    assert_main_thread!();

    let mut window_manager = WINDOW_MANAGER.write().unwrap();

    let Some(info) = window_manager.windows.remove(&id) else {
        log::warn!("Window {id} was already closed");
        return window_manager.windows.len();
    };

    window_manager
        .being_destroyed
        .insert(u64::from(info.native.id()));

    let removed = window_manager
        .winit_to_engine
        .remove(&info.native.id().into());

    assert!(removed.is_some(), "Native to engine ID map was missing");

    if info.is_primary {
        // Grab an arbitrary window to be the primary one, given that the user did not appoint one themself
        if let Some((_, window_info)) = window_manager.windows.iter_mut().next() {
            window_info.is_primary = true;
        }
    }

    window_manager.windows.len()
}

/// Notifies the window manager that a native [winit] window was fully destroyed
pub(crate) fn winit_window_destroyed(id: winit::window::WindowId) {
    profiling::function_scope!();

    assert_main_thread!();

    let mut window_manager = WINDOW_MANAGER.write().unwrap();

    let removed = window_manager.being_destroyed.remove(&u64::from(id));

    if !removed {
        log::error!("Destroyed winit window was not tracked as being destroyed. Internal error");
    }
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

/// Marks window `id` as the new primary window
pub(crate) fn appoint_primary_window(id: crate::window::Window) {
    profiling::function_scope!();

    let mut window_manager = WINDOW_MANAGER.write().unwrap();

    // We first set the new primary window directly on the window in question,
    // to make sure it still exists so that we are not left without a primary window
    // by accident
    let Some(window) = window_manager.windows.get_mut(&id) else {
        log::error!("Could not update icon for window {id} because it could not be found");
        return;
    };

    window.is_primary = true;

    // Now we unset all other windows to non-primary
    for (&win_id, window) in window_manager.windows.iter_mut() {
        if win_id != id {
            window.is_primary = false;
        }
    }
}

/// Refreshes the cached info for the given [Window]. Should be called by the main
/// engine runtime when a relevant [winit] event is received, or if some other window
/// attribute was changed. Also reconfigures the window surface if any attributes that influence
/// it have changed, or when `force_reconfigure` is `true`
///
/// Must be called from the main thread
pub(crate) fn refresh_window(id: &crate::window::Window, force_reconfigure: bool) {
    profiling::function_scope!();

    assert_main_thread!();

    log::trace!("Refreshing window info for {id}");

    let mut window_manager = WINDOW_MANAGER.write().unwrap();

    let Some(window) = window_manager.windows.get_mut(id) else {
        log::error!("Could not refresh cached info for window {id} because it could not be found");
        return;
    };

    let should_reconfigure = window.refresh();

    if force_reconfigure || should_reconfigure {
        log::debug!("Reconfiguring window surface for {id}");
        window.reconfigure_surface();
    }
}

/// Notifies the window manager that the occlusion status of window `id` has changed
pub(crate) fn notify_window_occluded(id: &crate::window::Window, occluded: bool) {
    profiling::function_scope!();

    assert_main_thread!();

    log::debug!("Changing occluded status of window {id} to {occluded}");

    let mut window_manager = WINDOW_MANAGER.write().unwrap();

    let Some(window) = window_manager.windows.get_mut(id) else {
        log::error!(
            "Could not change occluded status of window {id} to {occluded} because it could not be found"
        );
        return;
    };

    window.occluded = occluded;
}

/// Refreshes the known displays
pub(crate) fn refresh_displays(event_loop: &winit::event_loop::ActiveEventLoop) {
    profiling::function_scope!();
    assert_main_thread!();

    log::trace!("Refreshing display info");

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

pub(crate) fn request_redraw(window: Window) {
    profiling::function_scope!();
    assert_main_thread!();

    let window_manager = WINDOW_MANAGER.read().unwrap();

    if let Some(window) = window_manager.windows.get(&window) {
        window.native.request_redraw();
    }
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

/// The state of a native window
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum WindowState {
    /// Window is alive in WutEngine, and has a user-accessible ID
    Alive(Window),

    /// Window is being destroyed. It is no longer accessible by WutEngine users,
    /// but we're still waiting for winit to confirm destruction
    BeingDestroyed,

    /// Unknown window. Something has gone wrong
    NotFound,
}

/// For a given [winit::window::WindowId], returns the WutEngine [Window] if one can be found
pub(crate) fn find_id(native_id: winit::window::WindowId) -> WindowState {
    profiling::function_scope!();

    let window_manager = WINDOW_MANAGER.read().unwrap();

    if let Some(window) = window_manager.winit_to_engine.get(&native_id.into()) {
        return WindowState::Alive(*window);
    }

    if window_manager
        .being_destroyed
        .contains(&u64::from(native_id))
    {
        return WindowState::BeingDestroyed;
    }

    WindowState::NotFound
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
    assert_main_thread!();

    let window_manager = WINDOW_MANAGER.read().unwrap();

    for window in windows {
        if let Some(window_info) = window_manager.windows.get(window) {
            window_info.native.pre_present_notify();
            window_info.native.request_redraw();
        } else {
            log::warn!(
                "Could not pre-present notify window {window} because it could not be found"
            );
        }
    }
}

/// Locks the window manager, and calls `map_func` with a reference to the display info for `id`, if it exists.
///
/// The window manager is locked for the duration of the callback
#[inline(always)]
pub(super) fn get_display_and<T>(
    id: Display,
    map_func: impl FnOnce(Option<&DisplayInfo>) -> T,
) -> T {
    profiling::function_scope!();

    let window_manager = WINDOW_MANAGER.read().unwrap();

    map_func(window_manager.displays.get(&id))
}

pub(super) fn get_displays() -> Vec<Display> {
    WINDOW_MANAGER
        .read()
        .unwrap()
        .displays
        .keys()
        .copied()
        .collect()
}

pub(super) fn primary_display() -> Option<Display> {
    let window_manager = WINDOW_MANAGER.read().unwrap();

    if let Some(prim_id) = window_manager.primary_display {
        return Some(prim_id);
    }

    // Fallback: if we can't determine the primary display just return an arbitrary one
    window_manager.displays.keys().copied().next()
}

pub(super) fn monitor_handle_from_display(id: Display) -> Option<winit::monitor::MonitorHandle> {
    let window_manager = WINDOW_MANAGER.read().unwrap();

    window_manager
        .displays
        .get(&id)
        .map(|info| info.handle.clone())
}

fn unwrap_surface_tex(surface: &wgpu::Surface, window: Window) -> Option<wgpu::SurfaceTexture> {
    match surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Success(sfctex) => Some(sfctex),
        wgpu::CurrentSurfaceTexture::Suboptimal(sfctex) => {
            log::warn!("Suboptimal surface for window {window}, should recreate");
            crate::runtime::notify_event_loop(crate::runtime::WinitEvent::ForceSurfaceReconfigure(
                window,
            ));
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
            log::warn!("Surface texture for window {window} is outdated");
            crate::runtime::notify_event_loop(crate::runtime::WinitEvent::ForceSurfaceReconfigure(
                window,
            ));
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

    /// Winit windows currently being destroyed
    being_destroyed: IntSet<u64>,

    windows: IntMap<crate::window::Window, WindowInfo>,

    primary_display: Option<crate::window::Display>,
    displays: IntMap<crate::window::Display, DisplayInfo>,
}

#[derive(Debug)]
pub(super) struct WindowInfo {
    /// The engine-internal ID
    pub(super) id: Window,

    pub(super) title: String,

    /// Whether this is the primary window
    pub(super) is_primary: bool,

    /// The actual native window handle
    pub(super) native: Arc<winit::window::Window>,

    /// The rendering surface for the window
    pub(super) surface: wgpu::Surface<'static>,

    /// The physical window size, in pixels `W x H`
    pub(super) inner_size: (u32, u32),

    /// The OS-provided scale factor for the window
    pub(super) os_scale_factor: f64,

    /// Whether the window is currently focused
    pub(super) focused: bool,

    /// Whether the window is known to be occluded. Not supported
    /// by every OS, in which case this will always be `false`
    pub(super) occluded: bool,

    /// Whether the window is currently minimized
    pub(super) minimized: bool,

    /// Whether the window is currently maximized
    pub(super) maximized: bool,
}

impl WindowInfo {
    /// Creates a new [WindowInfo] struct, containing cached
    /// window information to prevent problems with querying information
    /// on non-main threads
    pub(crate) fn new(
        id: Window,
        title: String,
        is_primary: bool,
        native: Arc<winit::window::Window>,
        surface: wgpu::Surface<'static>,
    ) -> Self {
        let mut new = Self {
            id,
            title,
            is_primary,
            native,
            surface,
            inner_size: (0, 0),
            os_scale_factor: 1.0,
            focused: true,
            occluded: false,
            minimized: false,
            maximized: false,
        };

        let can_configure = new.refresh();

        if can_configure {
            new.reconfigure_surface();
        }

        new
    }

    /// Refresh all cached window information.
    ///
    /// Returns whether the surface should also be reconfigured
    fn refresh(&mut self) -> bool {
        assert_main_thread!();

        log::trace!("Refreshing cached information for window {}", self.id);

        self.title = self.native.title();

        let prev_inner_size = self.inner_size;

        self.inner_size = self.native.inner_size().into();
        self.os_scale_factor = self.native.scale_factor();

        let prev_focused = self.focused;
        self.focused = self.native.has_focus();

        if self.focused != prev_focused {
            log::debug!(
                "Window {} changed focus state to: {}",
                self.id,
                self.focused
            );
        }

        if let Some(minimized) = self.native.is_minimized() {
            self.minimized = minimized;
        }

        self.maximized = self.native.is_maximized();

        // We can't configure a 0-sized surface, so do not reconfigure if so
        self.inner_size != (0, 0) && prev_inner_size != self.inner_size
    }

    fn reconfigure_surface(&self) {
        log::debug!("Reconfiguring surface for window {}", self.id);

        let size = self.inner_size;

        if size == (0, 0) {
            log::error!("Cannot configure a size 0 surface. Internal error");
            return;
        }

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
            being_destroyed: IntSet::default(),
            windows: IntMap::default(),
            primary_display: None,
            displays: IntMap::default(),
        }
    }
}

#[derive(Debug)]
pub(super) struct DisplayInfo {
    pub(super) handle: winit::monitor::MonitorHandle,
    pub(super) name: Option<String>,
    pub(super) size: (u32, u32),
    pub(super) refresh_rate_millihertz: u32,
    pub(super) scaling_factor: f64,
    pub(super) video_modes: Vec<DisplayExclusiveFullscreenMode>,
    pub(super) is_primary: bool,
}

/// A video mode for [super::FullscreenMode::Exclusive]. Combines
/// a target display, a resolution, refresh rate, and an amount of bits-per-color
#[derive(Debug, Clone)]
pub struct DisplayExclusiveFullscreenMode(
    pub(super) Display,
    pub(super) winit::monitor::VideoModeHandle,
);

impl DisplayExclusiveFullscreenMode {
    /// The target display for this fullscreen mode
    #[inline]
    pub const fn display(&self) -> Display {
        self.0
    }

    /// The resolution of this mode
    #[inline]
    pub fn resolution(&self) -> (u32, u32) {
        let phys_size = self.1.size();

        (phys_size.width, phys_size.height)
    }

    /// The refresh rate in millihertz for this mode
    #[inline]
    pub fn refresh_rate_millihertz(&self) -> u32 {
        self.1.refresh_rate_millihertz()
    }

    /// The available bits per color for this mode
    #[inline]
    pub fn bits_per_color(&self) -> u16 {
        self.1.bit_depth()
    }
}

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
            refresh_rate_millihertz: handle
                .refresh_rate_millihertz()
                .expect("Display dissapeared during configuration"),
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

                let title = if info.is_primary {
                    format!("{} [Primary Window]", info.title)
                } else {
                    info.title.clone()
                };

                egui::CollapsingHeader::new(title)
                    .id_salt(*id)
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label(format!("ID: {id}"));
                        ui.label(format!("Size: {}x{}", info.inner_size.0, info.inner_size.1));
                        ui.label(format!("OS scale factor: {}", info.os_scale_factor));

                        ui.label(format!("Focused: {}", info.focused));
                        ui.label(format!("Occluded: {}", info.occluded));
                        ui.label(format!("Minimized: {}", info.minimized));
                        ui.label(format!("Maximized: {}", info.maximized));

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

                        if ui.button("Reconfigure").clicked() {
                            crate::runtime::notify_event_loop(
                                crate::runtime::WinitEvent::ForceSurfaceReconfigure(*id),
                            );
                        }
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
