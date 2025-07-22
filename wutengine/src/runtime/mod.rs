//! Main WutEngine runtime

use core::any::{Any, TypeId};
use std::sync::Mutex;

use rayon::prelude::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};
use wgpu::wgt::CommandEncoderDescriptor;
use wutengine_windowing::window::lock_windows;

use crate::component::{ComponentData, find_components};
use crate::prelude::{Camera, CameraTargetTexture, Component};
use crate::{component, gameobject, graphics, time};

/// Runs a single frame
pub(crate) fn run_step() {
    profiling::finish_frame!();
    profiling::function_scope!();

    log::trace!("Starting new frame");

    let fixed_updates = unsafe { time::update_to_now() };

    run_frame_phase("Update", || {
        component::run_on_active_components(|component, context| {
            component.on_update(context);
        });
    });

    {
        profiling::scope!("Fixed updates");

        for _ in 0..fixed_updates {
            run_frame_phase("Fixed update", || {
                component::run_on_active_components(|component, context| {
                    component.on_fixed_update(context);
                });
            });
        }
    }
}

fn run_frame_phase(_name: &'static str, phase: impl FnOnce()) {
    profiling::scope!(_name);

    component::add_queued();

    gameobject::handle_state_changes();
    component::handle_enable_disable();

    phase();

    loop {
        let any_handled = crate::event::handle_pending_events();

        if !any_handled {
            break;
        }
    }

    component::handle_destruction();
    gameobject::cleanup_destroyed();
}

#[profiling::function]
pub(crate) fn render() {
    log::trace!("Rendering frame");

    let mut cameras = Vec::with_capacity(10);

    component::get_components_of_type::<Camera>(&mut cameras);

    let mut renderers = Vec::with_capacity(512);

    find_components(
        |component| {
            component
                .implementation
                .lock()
                .unwrap()
                .as_renderer()
                .is_some()
        },
        &mut renderers,
    );

    log::trace!(
        "Found {} cameras and {} renderers",
        cameras.len(),
        renderers.len()
    );

    let renderers_vec: Vec<_> = renderers
        .iter()
        .map(|renderer| &renderer.implementation)
        .collect();

    let mut bufs_tgts: Vec<_> = Vec::with_capacity(cameras.len());

    {
        profiling::scope!("Make render command buffers for windows");

        cameras
            .par_iter()
            .map(|camera| render_commands_for_camera(camera, &renderers_vec))
            .collect_into_vec(&mut bufs_tgts);
    }

    let mut buffers = Vec::with_capacity(bufs_tgts.len());
    let mut targets = Vec::with_capacity(bufs_tgts.len());

    {
        profiling::scope!("Submit command buffers");

        bufs_tgts.into_iter().flatten().for_each(|(cmd_buf, tgt)| {
            buffers.push(cmd_buf);
            targets.push(tgt);
        });

        graphics::submit_command_buffers(buffers.into_iter());
    }

    {
        profiling::scope!("Swapping buffers");

        wutengine_windowing::window::lock_windows(|windows| {
            for (window_id, window) in windows {
                window.pre_present_notify();
            }

            for render_target in targets {
                match render_target {
                    CameraTargetTexture::Surface(surface_texture) => surface_texture.present(),
                }
            }
        });
    }
}

#[profiling::function]
fn render_commands_for_camera(
    camera: &ComponentData,
    renderers: &[&Mutex<Box<dyn Component>>],
) -> Option<(wgpu::CommandBuffer, CameraTargetTexture)> {
    let camera_component_locked = camera.implementation.lock().unwrap();

    let camera_component = (camera_component_locked.as_ref() as &dyn Any)
        .downcast_ref::<Camera>()
        .expect("Invalid cast");

    let camera_texture = camera_component.get_target_texture()?;

    let camera_texture_view = camera_texture.create_view(&wgpu::TextureViewDescriptor {
        format: Some(camera_texture.format()),
        label: Some(format!("Camera {} render target", camera.id).as_str()),
        ..Default::default()
    });

    let mut encoder = graphics::create_command_encoder(&CommandEncoderDescriptor {
        label: Some(format!("Camera {} command encoder", camera.id).as_str()),
    });

    {
        profiling::scope!("Record color pass");

        let mut color_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(format!("Camera {} color pass", camera.id).as_str()),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &camera_texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: match camera_component.get_background() {
                        crate::prelude::CameraBackground::None => wgpu::LoadOp::Load,
                        crate::prelude::CameraBackground::Color(color) => {
                            wgpu::LoadOp::Clear(color.into())
                        }
                    },
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }

    Some((encoder.finish(), camera_texture))
}
