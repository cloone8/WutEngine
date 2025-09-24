//! Main WutEngine runtime

use std::time::Instant;

use crate::prelude::SystemPhase;
use crate::{system, time};

pub mod renderer;
pub(crate) mod world;

pub(crate) use renderer::render_frame;

/// Runs a single frame
pub(crate) fn run_step() {
    profiling::finish_frame!();
    profiling::function_scope!();

    log::trace!("Starting new frame");

    let fixed_updates = time::update_frame(Instant::now());

    run_frame_phase("Update", || {
        system::run_systems_for_phase(SystemPhase::Update, &world::WORLD_MANAGER.shared());
    });

    {
        profiling::scope!("Fixed updates");

        for _ in 0..fixed_updates {
            run_frame_phase("Fixed update", || {
                system::run_systems_for_phase(
                    SystemPhase::FixedUpdate,
                    &world::WORLD_MANAGER.shared(),
                );

                time::update_fixed();
            });
        }
    }
}

fn run_frame_phase(_name: &'static str, phase: impl FnOnce()) {
    profiling::scope!(_name);

    world::run_spawn_queue();

    // gameobject::handle_state_changes();
    // component::handle_enable_disable();

    phase();

    // Handle any pending events, and then handle events published
    // while handling those events, etc.

    loop {
        let any_handled = crate::event::handle_pending_events();

        if !any_handled {
            break;
        }
    }

    // component::handle_destruction();
    // gameobject::cleanup_destroyed();
}

// #[profiling::function]
// pub(crate) fn render() {
//     log::trace!("Rendering frame");

//     let mut cameras = Vec::with_capacity(10);

// component::get_components_of_type::<Camera>(&mut cameras);

//     let mut renderers = Vec::with_capacity(512);

// find_components(
//     |component| {
//         component
//             .implementation
//             .lock()
//             .unwrap()
//             .as_renderer()
//             .is_some()
//     },
//     &mut renderers,
// );

//     log::trace!(
//         "Found {} cameras and {} renderers",
//         cameras.len(),
//         renderers.len()
//     );

//     let renderers_vec: Vec<_> = renderers
//         .iter()
//         .map(|renderer| &renderer.implementation)
//         .collect();

//     let mut bufs_tgts: Vec<_> = Vec::with_capacity(cameras.len());

//     {
//         profiling::scope!("Make render command buffers for windows");

//         cameras
//             .par_iter()
//             .map(|camera| render_commands_for_camera(camera, &renderers_vec))
//             .collect_into_vec(&mut bufs_tgts);
//     }

//     let mut buffers = Vec::with_capacity(bufs_tgts.len());
//     let mut targets = Vec::with_capacity(bufs_tgts.len());

//     {
//         profiling::scope!("Submit command buffers");

//         bufs_tgts.into_iter().flatten().for_each(|(cmd_buf, tgt)| {
//             buffers.push(cmd_buf);
//             targets.push(tgt);
//         });

//         graphics::submit_command_buffers(buffers.into_iter());
//     }

//     {
//         profiling::scope!("Swapping buffers");

//         wutengine_windowing::window::lock_windows(|windows| {
//             for (_window_id, window) in windows {
//                 window.pre_present_notify();
//             }

//             for render_target in targets {
//                 match render_target {
//                     CameraTargetTexture::Surface(surface_texture) => surface_texture.present(),
//                 }
//             }
//         });
//     }

//     // Mark any one-shot buffers we used this frame as available for the next frame
//     graphics::buffer::cache::recycle();
// }

// #[profiling::function]
// fn render_commands_for_camera(
//     camera: &ComponentData,
//     renderers: &[&Mutex<Box<dyn Component>>],
// ) -> Option<(wutengine_graphics::wgpu::CommandBuffer, CameraTargetTexture)> {
//     let mut camera_component_locked = camera.implementation.lock().unwrap();

//     let camera_component = (camera_component_locked.as_mut() as &mut dyn Any)
//         .downcast_mut::<Camera>()
//         .expect("Invalid cast");

//     camera_component.remake_view_mat();
//     camera_component.update_viewport_buffer();

//     let camera_texture = camera_component.get_target_texture()?;
//     let camera_texture_format = camera_texture.format();

//     let camera_texture_view =
//         camera_texture.create_view(&wutengine_graphics::wgpu::TextureViewDescriptor {
//             format: Some(camera_texture_format),
//             label: Some(format!("Camera {} render target", camera.id).as_str()),
//             ..Default::default()
//         });

//     let view_mat = camera_component.get_view_mat();
//     let projection_mat = camera_component.get_projection_mat();
//     let vp_map = projection_mat * view_mat;
//     let viewport_bind_group = camera_component.get_viewport_bind_group();

//     let mut encoder = graphics::create_command_encoder(&CommandEncoderDescriptor {
//         label: Some(format!("Camera {} command encoder", camera.id).as_str()),
//     });

//     {
//         profiling::scope!("Record color pass");

//         let mut color_pass =
//             encoder.begin_render_pass(&wutengine_graphics::wgpu::RenderPassDescriptor {
//                 label: Some(format!("Camera {} color pass", camera.id).as_str()),
//                 color_attachments: &[Some(wutengine_graphics::wgpu::RenderPassColorAttachment {
//                     view: &camera_texture_view,
//                     depth_slice: None,
//                     resolve_target: None,
//                     ops: wutengine_graphics::wgpu::Operations {
//                         load: match camera_component.get_background() {
//                             crate::prelude::CameraBackground::None => {
//                                 wutengine_graphics::wgpu::LoadOp::Load
//                             }
//                             crate::prelude::CameraBackground::Color(color) => {
//                                 wutengine_graphics::wgpu::LoadOp::Clear(color.into())
//                             }
//                         },
//                         store: wutengine_graphics::wgpu::StoreOp::Store,
//                     },
//                 })],
//                 depth_stencil_attachment: None,
//                 timestamp_writes: None,
//                 occlusion_query_set: None,
//             });

//         color_pass.set_bind_group(VIEWPORT_CONSTANTS_BIND_GROUP, viewport_bind_group, &[]);

//         for renderer in renderers {
//             let instance_constant_buffer =
//                 graphics::buffer::cache::get_and_write_buffer(&InstanceConstants {
//                     model_mat: Mat4::IDENTITY,
//                     mvp_mat: vp_map * Mat4::IDENTITY,
//                 });
//             let mut renderer = renderer.lock().unwrap();
//             let as_renderer = renderer.as_mut().as_renderer().expect("Given non-renderer");

//             as_renderer.render_color(&mut color_pass, camera_texture_format);
//         }
//     }

//     Some((encoder.finish(), camera_texture))
// }

// fn run_on_active_components(func: impl Fn(ComponentContext, &mut dyn Component) + Sync + Send) {
//     let gameobjects = GAMEOBJECT_MANAGER.gameobjects.read().unwrap();

//     gameobjects
//         .par_iter()
//         .filter(|(_, go)| go.is_enabled())
//         .for_each(|(id, go)| {
//             go.components
//                 .par_iter()
//                 .filter(|(_, component)| component.is_enabled())
//                 .for_each(|(component_id, component)| {
//                     let context = ComponentContext {
//                         gameobject: *id,
//                         this: *component_id,
//                     };

//                     let mut implementation = component.implementation.lock().unwrap();

//                     func(context, implementation.as_mut());
//                 });
//         });
// }
