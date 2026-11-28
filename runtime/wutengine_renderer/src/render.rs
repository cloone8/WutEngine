//! Main in-frame render functions

use wgpu::CommandEncoder;
use wgpu::CommandEncoderDescriptor;
use wgpu::LoadOp;
use wgpu::Operations;
use wgpu::RenderPassColorAttachment;
use wgpu::RenderPassDescriptor;
use wgpu::StoreOp;
use wutengine_math::Color;
use wutengine_sync::thread_context;

use crate::array_to_wgpu_color;
use crate::device;
use crate::label;
use crate::queue;
use crate::surfaces;
use crate::surfaces::AcquiredSurface;

/// Renders the current state of the renderer
pub fn render() {
    profiling::function_scope!();
    thread_context::assert_main_context();

    let surfaces = surfaces::acquire_surfaces();

    if surfaces.is_empty() {
        // Nothing to render to
        return;
    }

    let mut command_encoder = device().create_command_encoder(&CommandEncoderDescriptor {
        label: label!("WutEngine command encoder"),
    });

    record(&mut command_encoder, &surfaces);

    queue().submit([command_encoder.finish()]);

    for surface in surfaces {
        surface.present();
    }
}

fn record(encoder: &mut CommandEncoder, surfaces: &[AcquiredSurface]) {
    profiling::function_scope!();

    for surface in surfaces {
        let _ = encoder.begin_render_pass(&RenderPassDescriptor {
            label: label!("Surface {} clear pass", surface.id),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &surface.view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(array_to_wgpu_color(
                        Color::linear(0.5, 0.5, 0.5).to_srgb(),
                    )),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
}
