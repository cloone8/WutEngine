//! Module housing the window manager

use alloc::sync::Arc;
use std::sync::RwLock;

use nohash_hasher::IntMap;
use smallvec::SmallVec;

use crate::config;
use crate::graphics;
use crate::util::{InitOnce, assert_main_thread};

use super::Window;

/// The global [WindowManager]
static WINDOW_MANAGER: InitOnce<RwLock<WindowManager>> = InitOnce::new();

/// Initializes the global window management subsystem
pub(crate) fn initialize() {
    InitOnce::init(&WINDOW_MANAGER, RwLock::new(WindowManager::new()));
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

        surface.configure(
            graphics::device(),
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.0,
                height: size.1,
                present_mode: if config::try_get("wutengine.window.vsync").unwrap_or(true) {
                    wgpu::PresentMode::AutoVsync
                } else {
                    wgpu::PresentMode::AutoNoVsync
                },
                desired_maximum_frame_latency: if config::try_get(
                    "wutengine.window.triple_buffering",
                )
                .unwrap_or(false)
                {
                    2
                } else {
                    1
                },
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![
                    surface_format.remove_srgb_suffix(),
                    surface_format.add_srgb_suffix(),
                ],
            },
        );
    }
}

impl WindowManager {
    /// Creates a new default [WindowManager]
    pub(crate) fn new() -> Self {
        Self {
            winit_to_engine: IntMap::default(),
            windows: IntMap::default(),
        }
    }
}
