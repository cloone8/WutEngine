//! Camera components

use glam::{Mat4, Vec3};
use wutengine_core::identifiers::WindowIdentifier;
use wutengine_graphics::{color::Color, renderer::Viewport};

use crate::component::{Component, Context};
use crate::graphics;

use super::transform::Transform;

/// A camera that renders to a viewport in a window.
#[derive(Debug)]
pub struct Camera {
    /// The window to render to. Must match the identifier of an opened window in order
    /// for the camera to render anything.
    pub display: WindowIdentifier,

    /// The background color of the camera for any unset pixel
    pub clear_color: Color,

    /// The type of projection this camera draws, and the specific
    /// parameters of that projection
    pub camera_type: CameraType,
}

/// The different types of possible camera projections.
#[derive(Debug, Clone, Copy)]
pub enum CameraType {
    /// Perspective-projecting camera. Value defines vertical FOV in degrees
    Perspective(f64),

    /// Orthographic-projecting camera. Value defines vertical viewing volume.
    /// Horizontal volume is determined through aspect ratio
    Orthographic(f64),
}

#[profiling::all_functions]
impl Component for Camera {
    fn pre_render(&mut self, context: &mut Context) {
        let window = match context.window.get(&self.display) {
            Some(w) => w,
            None => {
                log::warn!(
                    "Camera trying to render to non-existing window {}",
                    self.display
                );
                return;
            }
        };

        let view_mat = match context.gameobject.get_component::<Transform>() {
            Some(t) => Mat4::look_to_lh(
                t.world_pos(),
                t.world_rot() * Vec3::Z,
                t.world_rot() * Vec3::Y,
            ),
            None => Mat4::IDENTITY,
        };

        let phys_window_size = window.size;

        let aspect_ratio: f64 = phys_window_size.0 as f64 / phys_window_size.1 as f64;

        let projection_mat = match self.camera_type {
            CameraType::Perspective(vertical_fov) => Mat4::perspective_lh(
                vertical_fov.to_radians() as f32,
                aspect_ratio as f32,
                0.1,
                100.0,
            ),
            CameraType::Orthographic(size) => {
                let half_size = size / 2.0;
                let half_horizontal_size = half_size * aspect_ratio;

                Mat4::orthographic_lh(
                    -half_horizontal_size as f32,
                    half_horizontal_size as f32,
                    -half_size as f32,
                    half_size as f32,
                    0.1,
                    100.0,
                )
            }
        };

        graphics::render_viewport(Viewport {
            window: self.display.clone(),
            clear_color: self.clear_color,
            view_mat,
            projection_mat,
        });
    }
}
