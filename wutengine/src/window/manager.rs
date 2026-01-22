//! Module housing the window manager

use nohash_hasher::IntMap;

use crate::assert_main_thread;

use super::WindowId;

/// The WutEngine window manager. Owns all native windows, and allows interaction with them
#[derive(Debug)]
pub(crate) struct WindowManager {
    /// Mapping from [winit] window IDs to WutEngine window IDs.
    /// use the [Into::into] implementation on [winit::window::WindowId] to convert
    /// it to a [u64] for use in this map
    winit_to_engine: IntMap<u64, crate::window::WindowId>,

    windows: IntMap<crate::window::WindowId, WindowInfo>,
}

#[derive(Debug)]
struct WindowInfo {
    /// The engine-internal ID
    id: WindowId,

    /// The actual native window handle
    native: winit::window::Window,

    /// The physical window size, in pixels `W x H`
    inner_size: (u32, u32),
}

impl WindowInfo {
    /// Creates a new [WindowInfo] struct, containing cached
    /// window information to prevent problems with querying information
    /// on non-main threads
    pub(crate) fn new(id: WindowId, native: winit::window::Window) -> Self {
        let mut new = Self {
            id,
            native,
            inner_size: (0, 0),
        };

        new.refresh();

        new
    }

    /// Refresh all cached window information
    fn refresh(&mut self) {
        assert_main_thread!("WindowInfo::refresh");

        log::trace!("Refreshing cached information for window {}", self.id);

        self.inner_size = self.native.inner_size().into();
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

    /// Registers a newly created window with the [WindowManager]
    ///
    /// Must be called from the main thread
    pub(crate) fn new_window(
        &mut self,
        id: crate::window::WindowId,
        window: winit::window::Window,
    ) {
        assert_main_thread!("WindowManager::new_window");

        let native_id = window.id();
        let info = WindowInfo::new(id, window);

        self.winit_to_engine.insert(native_id.into(), id);
        self.windows.insert(id, info);
    }

    /// Instructs the [WindowManager] to destroy the given window.
    ///
    /// If the window does not exist, does nothing.
    ///
    /// Must be called from the main thread
    pub(crate) fn close_window(&mut self, id: crate::window::WindowId) {
        assert_main_thread!("WindowManager::close_window");

        let Some(info) = self.windows.remove(&id) else {
            log::warn!("Window {id} was already closed");
            return;
        };

        let removed = self.winit_to_engine.remove(&info.native.id().into());
        assert!(removed.is_some(), "Native to engine ID map was missing");
    }

    /// Updates the icon of the given window to the new one.
    ///
    /// If the window does not exist, does nothing.
    ///
    /// Must be called from the main thread
    pub(crate) fn set_icon(&mut self, id: crate::window::WindowId, icon: winit::window::Icon) {
        let Some(window) = self.windows.get_mut(&id) else {
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
    pub(crate) fn refresh_cached_info(&mut self, id: &crate::window::WindowId) {
        assert_main_thread!("WindowManager::refresh_cached_info");

        log::trace!("Refreshing cached window info for {id}");

        let Some(window) = self.windows.get_mut(id) else {
            log::error!(
                "Could not refresh cached info for window {id} because it could not be found"
            );
            return;
        };

        window.refresh();
    }

    /// For a given [winit::window::WindowId], returns the WutEngine [WindowId] if one can be found
    pub(crate) fn find_id(&self, native_id: winit::window::WindowId) -> Option<WindowId> {
        self.winit_to_engine.get(&native_id.into()).copied()
    }
}
