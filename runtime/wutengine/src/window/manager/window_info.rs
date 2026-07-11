use alloc::sync::Arc;

use wutengine_graphics::wgpu;
use wutengine_util::assert_main_thread;

use crate::config;
use crate::graphics;
use crate::window::Window;

#[derive(Debug)]
pub(in crate::window) struct WindowInfo {
    /// The engine-internal ID
    pub(in crate::window) id: Window,

    pub(in crate::window) title: String,

    /// Whether this is the primary window
    pub(in crate::window) is_primary: bool,

    /// The actual native window handle
    pub(in crate::window) native: Arc<winit::window::Window>,

    /// The rendering surface for the window
    pub(in crate::window) surface: wgpu::Surface<'static>,

    /// The physical window size, in pixels `W x H`
    pub(in crate::window) inner_size: (u32, u32),

    /// The OS-provided scale factor for the window
    pub(in crate::window) os_scale_factor: f64,

    /// Whether the window is currently focused
    pub(in crate::window) focused: bool,

    /// Whether the window is known to be occluded. Not supported
    /// by every OS, in which case this will always be `false`
    pub(in crate::window) occluded: bool,

    /// Whether the window is currently minimized
    pub(in crate::window) minimized: bool,

    /// Whether the window is currently maximized
    pub(in crate::window) maximized: bool,
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
    pub(super) fn refresh(&mut self) -> bool {
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

    pub(super) fn reconfigure_surface(&self) {
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
            .find(|f| f.is_srgb() && !f.is_compressed())
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
                color_space: wgpu::SurfaceColorSpace::Srgb,
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
        if capabilities.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else if capabilities.contains(&wgpu::PresentMode::Immediate) {
            wgpu::PresentMode::Immediate
        } else {
            wgpu::PresentMode::Fifo
        }
    }
}
