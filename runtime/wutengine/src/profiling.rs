//! Profiling related functionality

pub use profiling::*;

#[cfg(feature = "profiling")]
mod internal {

    #[cfg(feature = "development_overlay")]
    pub(super) static DEV_OVERLAY_WINDOW_OPEN: core::sync::atomic::AtomicBool =
        core::sync::atomic::AtomicBool::new(false);

    #[cfg(feature = "development_overlay")]
    pub(super) fn dev_overlay_open() -> bool {
        DEV_OVERLAY_WINDOW_OPEN.load(core::sync::atomic::Ordering::Acquire)
    }
}

/// Sets the profiling scopes either on or off depending on whether something is listening
pub(crate) fn change_scope_active_status() {
    #[cfg(feature = "profiling")]
    {
        let overlay_active = cfg_select! {
            feature = "development_overlay" => {
                internal::dev_overlay_open()
            }
            _ => false
        };

        puffin::set_scopes_on(overlay_active);
    }
}

#[cfg(feature = "development_overlay")]
pub(crate) mod development_overlay {
    use wutengine_development_overlay::DevelopmentOverlayWindow;

    #[derive(Debug, Default)]
    pub(crate) struct ProfilingOverlay;

    impl DevelopmentOverlayWindow for ProfilingOverlay {
        fn name(&self) -> &str {
            "Profiler"
        }

        fn icon(&self) -> Option<&str> {
            Some("🚀")
        }

        #[cfg(not(feature = "profiling"))]
        fn show(&mut self, ui: &mut wutengine_development_overlay::wutengine_egui::egui::Ui) {
            ui.label("Profiling not enabled in build");
        }

        #[cfg(feature = "profiling")]
        fn show(&mut self, ui: &mut wutengine_development_overlay::wutengine_egui::egui::Ui) {
            wutengine_puffin_egui::profiler_ui(ui);
        }

        #[cfg(feature = "profiling")]
        fn window_state_changed(&mut self, opened: bool) {
            super::internal::DEV_OVERLAY_WINDOW_OPEN
                .store(opened, core::sync::atomic::Ordering::Release);
        }
    }
}
