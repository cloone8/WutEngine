use core::fmt::Display;

use serde::{Deserialize, Serialize};
use wutengine_graphics::color::Color;
use wutengine_graphics::wgpu::{self, Extent3d};
use wutengine_math::Mat4;
use wutengine_windowing::window::WindowIdentifier;

use crate::prelude::{Component, Entity, Name, SystemPhase};
use crate::system::register_system;

#[cfg(test)]
mod test;

mod config;
mod formats;

pub use config::*;
pub use formats::*;

/// A camera, rendering the scene from its viewport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    target: Option<CameraTarget>,
    projection: CameraProjection,
    background: CameraBackground,
    viewport: CameraViewport,
    clipping_planes: (f32, f32),
    color_format: CameraColorFormat,
    depth_stencil_format: CameraDepthStencilFormat,

    #[serde(skip)]
    color_tex: Option<wgpu::Texture>,

    #[serde(skip)]
    depth_tex: Option<wgpu::Texture>,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Camera {
    fn add_default_systems()
    where
        Self: Sized,
    {
        register_system::<(&mut Self, Option<&Name>)>(
            |entity, (camera, name)| camera.on_pre_render(name, entity),
            SystemPhase::PreRender,
        );
    }
}

/// Private API for [Camera] components
impl Camera {
    fn on_pre_render(&mut self, cam_name: Option<&Name>, _attached_entity: Entity) {
        let Some(target) = &self.target else {
            // No render target, nothing to do except make sure the color and depth textures are freed
            self.color_tex = None;
            self.depth_tex = None;
            return;
        };

        let target_inner_size = match target {
            CameraTarget::Window(win_id) => {
                let size = crate::window::window_size(win_id);

                match size {
                    Some(size) => size,
                    None => {
                        // Nothing to do, we cannot render to an unknown target
                        return;
                    }
                }
            }
        };

        let expected_target_size =
            Self::calc_internal_texture_size(target_inner_size, self.viewport);

        let should_recreate_color =
            Self::should_recreate_internal_tex(self.color_tex.as_ref(), expected_target_size);
        let should_recreate_depthstencil =
            Self::should_recreate_internal_tex(self.depth_tex.as_ref(), expected_target_size);

        if should_recreate_color {
            self.recreate_color_tex(
                expected_target_size,
                cam_name.map(|cn| cn.0.as_ref()).unwrap_or("<unknown>"),
            );
            assert!(self.color_tex.is_some());
        }

        if should_recreate_depthstencil {
            self.recreate_depthstencil_tex(
                expected_target_size,
                cam_name.map(|cn| cn.0.as_ref()).unwrap_or("<unknown>"),
            );
            assert!(self.depth_tex.is_some());
        }
    }

    fn recreate_color_tex(&mut self, size: (u32, u32), cam_name: &str) {
        log::info!("Recreating color texture on camera {cam_name}");

        let device = wutengine_graphics::raw_device();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("Camera {cam_name} Color Target")),
            size: Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::from(self.color_format),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        self.color_tex = Some(texture);
    }

    fn recreate_depthstencil_tex(&mut self, size: (u32, u32), cam_name: &str) {
        log::info!("Recreating depth/stencil texture on camera {cam_name}");

        let device = wutengine_graphics::raw_device();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("Camera {cam_name} DepthStencil Target")),
            size: Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::from(self.depth_stencil_format),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        self.depth_tex = Some(texture);
    }

    fn should_recreate_internal_tex(
        tex: Option<&wgpu::Texture>,
        expected_size: (u32, u32),
    ) -> bool {
        match tex {
            Some(tex) => (tex.size().width, tex.size().height) != expected_size,
            None => true,
        }
    }

    fn calc_internal_texture_size(
        target_inner_size: (u32, u32),
        viewport_size: CameraViewport,
    ) -> (u32, u32) {
        //TODO: Round or truncate?
        (
            (target_inner_size.0 as f32 * viewport_size.w).round() as u32,
            (target_inner_size.1 as f32 * viewport_size.h).round() as u32,
        )
    }

    fn remake_proj_mat(&mut self) {
        let phys_target_size = match self.target.as_ref() {
            Some(CameraTarget::Window(window_id)) => {
                let Some(phys_target_size) = crate::window::window_size(window_id) else {
                    log::warn!(
                        "Not recalculating projection matrix because the camera has an unknown window target: {window_id}"
                    );
                    return;
                };

                phys_target_size
            }
            None => {
                log::warn!(
                    "Not recalculating projection matrix because the camera does not have a target"
                );
                return;
            }
        };

        let aspect_ratio: f32 = phys_target_size.0 as f32 / phys_target_size.1 as f32;

        let projection_mat = match self.projection {
            CameraProjection::Perspective(fov) => Mat4::perspective_lh(
                fov.get_vertical(aspect_ratio).to_radians() as f32,
                aspect_ratio as f32,
                self.clipping_planes.0,
                self.clipping_planes.1,
            ),
            CameraProjection::Orthographic(size) => {
                let half_size = size / 2.0;
                let half_horizontal_size = half_size * aspect_ratio;

                Mat4::orthographic_lh(
                    -half_horizontal_size,
                    half_horizontal_size,
                    -half_size,
                    half_size,
                    self.clipping_planes.0,
                    self.clipping_planes.1,
                )
            }
        };

        todo!("Do something with matrix");
    }
}

/// Public API for [Camera]
impl Camera {
    pub fn new() -> Self {
        Self {
            target: None,
            projection: CameraProjection::Perspective(FieldOfView::Vertical(70.0)),
            background: CameraBackground::Color(Color::BLACK),
            viewport: CameraViewport::FULL_WINDOW,
            clipping_planes: (0.1, 100.0),
            color_format: CameraColorFormat::Rgba32Float,
            depth_stencil_format: CameraDepthStencilFormat::Depth32Float,
            color_tex: None,
            depth_tex: None,
        }
    }

    pub fn set_projection(&mut self, projection: CameraProjection) {
        self.projection = projection;
        self.remake_proj_mat();
    }

    pub fn set_window(&mut self, target: CameraTarget) {
        self.target = Some(target);
        self.remake_proj_mat();
    }

    pub fn set_background(&mut self, background: CameraBackground) {
        self.background = background;
    }

    pub fn set_viewport(&mut self, viewport: CameraViewport) {
        if !viewport.is_valid() {
            log::error!("Cannot set invalid viewport {viewport}");
            return;
        }

        self.viewport = viewport;
    }

    pub fn set_clipping_planes(&mut self, near: f32, far: f32) {
        if !near.is_normal() {
            log::error!("Cannot set near plane to invalid value: {near}");
            return;
        }

        if !far.is_normal() {
            log::error!("Cannot set far plane to invalid value: {far}");
            return;
        }

        if far <= near {
            log::error!("Far plane ({far}) closer than near plane ({near})");
            return;
        }

        self.clipping_planes = (near, far);
        self.remake_proj_mat();
    }

    /// Returns the current view matrix for this [Camera]
    #[inline(always)]
    pub fn get_view_mat(&self) -> Mat4 {
        todo!()
    }

    /// Returns the current projection matrix for this [Camera]
    #[inline(always)]
    pub fn get_projection_mat(&self) -> Mat4 {
        todo!()
    }

    /// Returns the current clear background for this [Camera]
    #[inline(always)]
    pub fn get_background(&self) -> CameraBackground {
        self.background
    }

    /// Returns the color target texture in this frame, if present.
    /// Note that this might not be valid after the current frame phase ends, as
    /// it might get recreated in [SystemPhase::PreRender] if the texture is incorrectly sized
    #[inline(always)]
    pub fn get_color_target(&self) -> Option<&wgpu::Texture> {
        self.color_tex.as_ref()
    }

    /// Returns the depth/stencil target texture in this frame, if present.
    /// Note that this might not be valid after the current frame phase ends, as
    /// it might get recreated in [SystemPhase::PreRender] if the texture is incorrectly sized
    #[inline(always)]
    pub fn get_depthstencil_target(&self) -> Option<&wgpu::Texture> {
        self.depth_tex.as_ref()
    }
}
