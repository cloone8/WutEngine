use wutengine_shadercompiler::MATERIAL_PARAMS_BIND_GROUP_INDEX;

use crate::builtins::components::rendering::Camera;
use crate::graphics;
use crate::graphics::DrawCommand;

use super::RenderPass;

/// The main pass for color rendering
#[derive(Debug, Clone)]
pub struct ColorPass;

impl ColorPass {
    /// The position relative to other renderpasses. Higher is later
    pub const ORDER: u64 = u64::MAX / 2;
}

impl RenderPass for ColorPass {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Color"
    }

    fn order() -> u64 {
        Self::ORDER
    }

    fn construct() -> Box<dyn RenderPass>
    where
        Self: Sized,
    {
        Box::new(ColorPass)
    }

    fn execute(
        &mut self,
        cmd: &mut wgpu::CommandEncoder,
        camera: &Camera,
        draw_commands: &[DrawCommand],
    ) {
        log::trace!("Running color pass");

        let Some(target_tex) = camera.get_render_target() else {
            log::error!("Failed to execute color pass because the render target was missing");
            return;
        };

        let target_view = target_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let color_targets = [Some(wgpu::ColorTargetState {
            format: target_tex.format(),
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let mut render_pass = cmd.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Color"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: camera.background.to_wgpu_load_op(),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        if let Err(()) = camera.set_camera_bind_group_on_pass(&mut render_pass) {
            log::error!("Failed to set camera bind group");
            return;
        }

        let mut cur_pipeline = None;
        let mut cur_material = None;
        let mut cur_topology = None;

        'drawcall: for draw_command in draw_commands {
            if let Some(target_cam) = draw_command.camera
                && target_cam != camera.get_id()
            {
                continue;
            }

            render_pass.push_debug_group("Draw command");

            let material_changed =
                cur_material.is_none() || cur_material.unwrap() != draw_command.material.id;
            let mesh_topology_changed =
                cur_topology.is_none() || cur_topology.unwrap() != draw_command.mesh.topology();

            if material_changed {
                cur_material = Some(draw_command.material.id);
                render_pass.insert_debug_marker("Switch material");

                let Some(bind_group) = draw_command.material.user_bind_group.get_bind_group()
                else {
                    log::error!("Material user bind group out of date, cannot execute draw call");
                    render_pass.pop_debug_group();
                    continue;
                };

                render_pass.set_bind_group(MATERIAL_PARAMS_BIND_GROUP_INDEX, bind_group, &[]);
            }

            if mesh_topology_changed {
                cur_topology = Some(draw_command.mesh.topology());
            }

            if material_changed || mesh_topology_changed {
                // Pipeline check is slightly more expensive, so we only retrieve a new pipeline if any
                // of the things that might change it have changed
                let pipeline = match graphics::pipeline::get_pipeline(
                    &draw_command.material,
                    draw_command.mesh.topology(),
                    &color_targets,
                ) {
                    Ok(pipeline) => pipeline,
                    Err(e) => {
                        log::error!("Failed to get pipeline for draw call: {e}");
                        render_pass.pop_debug_group();
                        continue;
                    }
                };

                if cur_pipeline.is_none() || cur_pipeline.as_ref().unwrap() != &pipeline {
                    render_pass.insert_debug_marker("Switch pipeline");
                    render_pass.set_pipeline(&pipeline);
                    cur_pipeline = Some(pipeline);
                }
            }

            // Do the actual draw call for this command

            //TODO: Set instance bind group

            let attrs = &draw_command.material.compiled_shader.vertex_attributes;

            for (attribute, &location) in attrs {
                let Some(vertex_buffer) = draw_command.mesh.vertex_buffers.get(attribute) else {
                    log::error!(
                        "Mesh is missing vertex buffer for requested attribute: {attribute}"
                    );
                    render_pass.pop_debug_group();
                    continue 'drawcall;
                };

                render_pass.set_vertex_buffer(location, vertex_buffer.buffer.slice(..));
            }

            render_pass.set_index_buffer(
                draw_command.mesh.index_buffer.buffer.slice(..),
                draw_command.mesh.index_buffer.format.to_wgpu(),
            );

            render_pass.draw_indexed(0..draw_command.mesh.index_buffer.count as u32, 0, 0..1);

            render_pass.pop_debug_group();
        }
    }
}
