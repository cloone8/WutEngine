//! Module housing the window manager

use alloc::sync::Arc;
use core::sync::atomic::AtomicUsize;
use display_info::DisplayInfo;
use nohash_hasher::IntSet;
use std::sync::RwLock;
use window_info::WindowInfo;
use wutengine_graphics::wgpu;

use nohash_hasher::IntMap;
use smallvec::SmallVec;

use crate::config;
use wutengine_util::InitOnce;
use wutengine_util::assert_main_thread;

use super::Display;
use super::Window;
use super::WindowUpdateEvent;

#[cfg(feature = "development_overlay")]
mod development_overlay;

pub(super) mod display_info;
pub(super) mod window_info;

/// The global [`WindowManager`]
static WINDOW_MANAGER: InitOnce<RwLock<WindowManager>> = InitOnce::new_checked();

/// Initializes the global window management subsystem
pub(crate) fn init() {
    InitOnce::init(&WINDOW_MANAGER, RwLock::new(WindowManager::new()));

    #[cfg(feature = "development_overlay")]
    {
        crate::development_overlay::add_development_overlay_window(
            development_overlay::WindowManagerOverlay::default(),
        );
    }

    wutengine_event::subscribe::<config::ConfigValueChanged>(|changed| {
        match changed.key.as_str() {
            "wutengine.window.vsync" | "wutengine.window.triple_buffering" => {
                get_windows_and(|window_map| {
                    for window in window_map.keys() {
                        crate::runtime::send_to_main_thread(
                            crate::runtime::MainThreadEvent::ForceSurfaceReconfigure(*window),
                        );
                    }
                });
            }
            _ => {}
        }
    });
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
#[inline]
pub(super) fn get_windows_and<T>(map_func: impl FnOnce(&IntMap<Window, WindowInfo>) -> T) -> T {
    profiling::function_scope!();

    let window_manager = WINDOW_MANAGER.read().unwrap();

    map_func(&window_manager.windows)
}

/// Locks the window manager, and calls `map_func` with a reference to the window info for `id`, if it exists.
///
/// The window manager is locked for the duration of the callback
#[inline]
pub(super) fn get_window_and<T>(id: Window, map_func: impl FnOnce(Option<&WindowInfo>) -> T) -> T {
    profiling::function_scope!();

    let window_manager = WINDOW_MANAGER.read().unwrap();

    map_func(window_manager.windows.get(&id))
}

/// Returns the amount of currently open windows
pub(crate) fn num_windows() -> usize {
    profiling::function_scope!();

    assert_main_thread!();

    WINDOW_MANAGER.read().unwrap().windows.len()
}

/// Instructs the [`WindowManager`] to destroy the given window.
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

/// Notifies the window manager that a native [`winit`] window was fully destroyed
pub(crate) fn winit_window_destroyed(id: winit::window::WindowId) {
    profiling::function_scope!();

    assert_main_thread!();

    let mut window_manager = WINDOW_MANAGER.write().unwrap();

    let removed = window_manager.being_destroyed.remove(&u64::from(id));

    if !removed {
        log::error!("Destroyed winit window was not tracked as being destroyed. Internal error");
    }
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

/// Refreshes the cached info for the given [`Window`]. Should be called by the main
/// engine runtime when a relevant [`winit`] event is received, or if some other window
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

/// For a given [winit::window::WindowId], returns the WutEngine [`Window`] if one can be found
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
    profiling::scope!("Wait on vsync");

    let window_manager = WINDOW_MANAGER.read().unwrap();

    let mut surfaces: SmallVec<[_; 2]> = SmallVec::new_const();

    for (&id, window_info) in window_manager.windows.iter() {
        profiling::scope!("Get surface texture", &id.to_string());
        log::trace!("Getting surface texture for window {id}");

        if let Some(surface_tex) = unwrap_surface_tex(&window_info.surface, id) {
            surfaces.push((id, surface_tex));
        }
    }

    surfaces
}

/// Request redraws for all available windows
pub(crate) fn request_redraws() {
    profiling::function_scope!();
    assert_main_thread!();

    let window_manager = WINDOW_MANAGER.read().unwrap();

    for window in window_manager.windows.values() {
        window.native.request_redraw();
    }
}

/// Notifies the given windows that content is about to be presented to them
pub(crate) fn pre_present_notify<'a>(windows: impl IntoIterator<Item = &'a Window>) {
    profiling::function_scope!();
    assert_main_thread!();

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

/// Updates the window according to the given event. If the window does not exist, does nothing.
/// Must be called from the main thread
pub(crate) fn handle_update(target: Window, event: WindowUpdateEvent) {
    assert_main_thread!();

    let window_manager = WINDOW_MANAGER.read().unwrap();

    let Some(window) = window_manager.windows.get(&target) else {
        log::error!("Could not update icon for window {target} because it could not be found");
        return;
    };

    match event {
        WindowUpdateEvent::UpdateIcon(icon) => {
            profiling::scope!("Update Icon");

            log::debug!("Handling icon update request for window {target}");

            #[cfg(windows)]
            {
                use winit::platform::windows::WindowExtWindows;

                window.native.set_taskbar_icon(Some(icon.clone()));
            }

            window.native.set_window_icon(Some(icon));
        }
        WindowUpdateEvent::UpdateTitle(title) => {
            profiling::scope!("Update Title");

            window.native.set_title(title.as_str());
        }
        WindowUpdateEvent::UpdateCursor(cursor) => {
            profiling::scope!("Update Cursor");

            window.native.set_cursor(cursor);
        }
        WindowUpdateEvent::CursorVisibility(show) => {
            profiling::scope!("Cursor Visibility");

            window.native.set_cursor_visible(show);
        }
    }
}

/// Locks the window manager, and calls `map_func` with a reference to the display info for `id`, if it exists.
///
/// The window manager is locked for the duration of the callback
#[inline]
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
            crate::runtime::send_to_main_thread(
                crate::runtime::MainThreadEvent::ForceSurfaceReconfigure(window),
            );
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
            crate::runtime::send_to_main_thread(
                crate::runtime::MainThreadEvent::ForceSurfaceReconfigure(window),
            );
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
    /// Mapping from [`winit`] window IDs to WutEngine window IDs.
    /// use the [Into::into] implementation on [`winit::window::WindowId`] to convert
    /// it to a [`u64`] for use in this map
    winit_to_engine: IntMap<u64, crate::window::Window>,

    /// Winit windows currently being destroyed
    being_destroyed: IntSet<u64>,

    windows: IntMap<crate::window::Window, WindowInfo>,

    primary_display: Option<crate::window::Display>,
    displays: IntMap<crate::window::Display, DisplayInfo>,
}

impl WindowManager {
    /// Creates a new default [`WindowManager`]
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
