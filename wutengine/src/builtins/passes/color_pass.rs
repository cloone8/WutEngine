use alloc::sync::Arc;

use wutengine_asset::assets::mesh::MeshTopology;
use wutengine_graphics::material::Material;
use wutengine_graphics::material::MaterialId;
use wutengine_graphics::mesh::Mesh;
use wutengine_graphics::renderpass::RenderPass;
use wutengine_graphics::wgpu;
use wutengine_shadercompiler::INSTANCE_PARAMS_BIND_GROUP_INDEX;
use wutengine_shadercompiler::MATERIAL_PARAMS_BIND_GROUP_INDEX;

use crate::builtins::components::rendering::Camera;
use crate::graphics;
use crate::graphics::DrawCommand;

/// The main pass for color rendering
#[derive(Debug, Clone)]
pub struct ColorPass;

impl ColorPass {
    /// The position relative to other renderpasses. Higher is later
    pub const ORDER: u64 = u64::MAX / 2;
}

impl RenderPass<Camera, DrawCommand> for ColorPass {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Color"
    }

    fn order() -> u64 {
        Self::ORDER
    }

    fn construct() -> Box<dyn RenderPass<Camera, DrawCommand>>
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
        profiling::function_scope!();

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

        if let Err(e) = camera.set_camera_bind_group_on_pass(&mut render_pass) {
            log::error!("Failed to set camera bind group: {e}");
            return;
        }

        let mut render_state = RenderState::default();

        for draw_command in draw_commands.iter() {
            if let Some(target_cam) = draw_command.camera
                && target_cam != camera.get_id()
            {
                continue;
            }

            render_pass.push_debug_group("Draw command");
            render_state.draw_single(&mut render_pass, draw_command, camera, &color_targets);
            render_pass.pop_debug_group();
        }
    }
}

#[derive(Debug, Default)]
struct RenderState {
    material: Option<MaterialId>,
    topology: Option<MeshTopology>,
    pipeline: Option<Arc<wgpu::RenderPipeline>>,
    draw_index: usize,
}

impl RenderState {
    fn update(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        next_material: &Material,
        next_mesh: &Mesh,
        color_targets: &[Option<wgpu::ColorTargetState>],
    ) -> Result<(), ()> {
        let material_changed =
            self.material.is_none() || self.material.unwrap() != next_material.id();
        let mesh_topology_changed =
            self.topology.is_none() || self.topology.unwrap() != next_mesh.topology();

        if material_changed {
            self.material = Some(next_material.id());
            render_pass.insert_debug_marker("Switch material");

            let Some(bind_group) = next_material.raw_bind_group().get_bind_group() else {
                log::error!("Material user bind group out of date, cannot execute draw call");
                return Err(());
            };

            render_pass.set_bind_group(MATERIAL_PARAMS_BIND_GROUP_INDEX, bind_group, &[]);
        }

        if mesh_topology_changed {
            self.topology = Some(next_mesh.topology());
        }

        if material_changed || mesh_topology_changed {
            // Pipeline check is slightly more expensive, so we only retrieve a new pipeline if any
            // of the things that might change it have changed
            let pipeline = match graphics::pipeline::get_pipeline(
                next_material,
                next_mesh.topology(),
                color_targets,
            ) {
                Ok(pipeline) => pipeline,
                Err(e) => {
                    log::error!("Failed to get pipeline for draw call: {e}");
                    return Err(());
                }
            };

            if self.pipeline.is_none() || self.pipeline.as_ref().unwrap() != &pipeline {
                render_pass.insert_debug_marker("Switch pipeline");
                render_pass.set_pipeline(&pipeline);
                self.pipeline = Some(pipeline);
            }
        }

        Ok(())
    }

    fn draw_single(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        draw_command: &DrawCommand,
        camera: &Camera,
        color_targets: &[Option<wgpu::ColorTargetState>],
    ) {
        let this_draw_index = self.draw_index;
        self.draw_index += 1;

        if let Err(()) = self.update(
            render_pass,
            &draw_command.material,
            &draw_command.mesh,
            color_targets,
        ) {
            return;
        };

        let queue = graphics::queue();
        let device = graphics::device();

        // Do the actual draw call for this command
        let mut instance_bind_group = graphics::internal_bind_groups::create_instance_bind_group(
            format!("Instance bind group draw call {this_draw_index}"),
        );

        if let Err(e) =
            instance_bind_group.set_parameter("model", draw_command.transform.into(), queue)
        {
            log::error!("Failed to set model matrix: {e}");
            return;
        }

        let mvp = camera.get_proj_mat() * camera.get_view_mat() * draw_command.transform;

        if let Err(e) = instance_bind_group.set_parameter("mvp", mvp.into(), queue) {
            log::error!("Failed to set mvp matrix: {e}");
            return;
        }

        instance_bind_group.update_bind_group(device);

        render_pass.set_bind_group(
            INSTANCE_PARAMS_BIND_GROUP_INDEX,
            instance_bind_group.get_bind_group(),
            &[],
        );

        let attrs = &draw_command.material.compiled_shader().vertex_attributes;

        for (attr_type, attr_info) in attrs {
            let Some(vertex_buffer) = draw_command.mesh.vertex_buffers.get(attr_type) else {
                log::error!("Mesh is missing vertex buffer for requested attribute: {attr_type}");
                return;
            };

            render_pass.set_vertex_buffer(attr_info.shader_location, vertex_buffer.raw().slice(..));
        }

        render_pass.set_index_buffer(
            draw_command.mesh.index_buffer.raw().slice(..),
            draw_command.mesh.index_buffer.format().to_wgpu(),
        );

        render_pass.draw_indexed(
            0..draw_command.mesh.index_buffer.len().get() as u32,
            0,
            0..1,
        );
    }
}
