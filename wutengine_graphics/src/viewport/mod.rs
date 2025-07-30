//! Viewport buffers and functionality

use core::num::NonZero;
use core::sync::atomic::{AtomicU32, Ordering};

use wgpu::wgt::BufferDescriptor;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BufferUsages};

use crate::GRAPHICS_MANAGER;
use crate::debug::debug_label;
use crate::shader::ShaderConstants;
use crate::shader::constants::ViewportConstants;

#[derive(Debug)]
pub struct Viewport {
    viewport_const_buffer: wgpu::Buffer,
    viewport_const_bind_group: wgpu::BindGroup,
    // instance_const_buffer: wgpu::Buffer,
    // instance_const_bind_group: wgpu::BindGroup,
}

/// Internal [Viewport] API
impl Viewport {}

/// Public [Viewport] API
impl Viewport {
    pub fn new(init_data: Option<&ViewportConstants>) -> Self {
        static VIEWPORT_ID: AtomicU32 = AtomicU32::new(0);
        let new_viewport_id = VIEWPORT_ID.fetch_add(1, Ordering::Relaxed);

        // Allocate new buffer and fill with data
        let new_buffer = GRAPHICS_MANAGER.device.create_buffer(&BufferDescriptor {
            label: debug_label!("Viewport {} constant buffer", new_viewport_id),
            size: size_of::<ViewportConstants>() as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: true,
        });

        let mut buffer_view = new_buffer.get_mapped_range_mut(..);

        if let Some(init_data) = init_data {
            buffer_view.copy_from_slice(bytemuck::bytes_of(init_data));
        } else {
            buffer_view.copy_from_slice(bytemuck::bytes_of(&ViewportConstants::IDENTITY));
        }

        drop(buffer_view);
        new_buffer.unmap();

        let new_bind_group = GRAPHICS_MANAGER
            .device
            .create_bind_group(&BindGroupDescriptor {
                label: debug_label!("Viewport {} bind group", new_viewport_id),
                layout: &ShaderConstants::viewport_bind_group_layout(),
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: new_buffer.as_entire_binding(),
                }],
            });

        Self {
            viewport_const_buffer: new_buffer,
            viewport_const_bind_group: new_bind_group,
        }
    }

    pub fn update(&self, data: &ViewportConstants) {
        let mut view = GRAPHICS_MANAGER
            .queue
            .write_buffer_with(
                &self.viewport_const_buffer,
                0,
                NonZero::new(size_of::<ViewportConstants>() as u64).unwrap(),
            )
            .expect("Could not obtain write view for viewport buffer");

        view.copy_from_slice(bytemuck::bytes_of(data));
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.viewport_const_bind_group
    }
}
