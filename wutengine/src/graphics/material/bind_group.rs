use core::num::NonZero;
use std::collections::HashMap;

use crate::graphics::shader::{NativeShaderBufferParameter, NativeShaderOpaqueParameter};
use crate::graphics::{GFX_DEVICE, GFX_QUEUE};

use super::MaterialParameter;

#[derive(Debug, Clone)]
pub(crate) struct BindGroup {
    bind_group_name: String,
    param_indices: HashMap<String, ParamIndex>,
    buffer_params: Vec<NativeShaderBufferParameter>,
    opaque_params: Vec<NativeShaderOpaqueParameter>,
    layout: wgpu::BindGroupLayout,
    native: Option<(wgpu::Buffer, wgpu::BindGroup)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ParamIndex {
    Buffer(u16),
    Opaque(u16),
}

#[derive(Debug)]
pub enum SetParamErr {
    UnknownParameter,
    InvalidConversion(&'static str, &'static str),
}

impl BindGroup {
    fn total_buffer_align(&self) -> usize {
        self.buffer_params
            .iter()
            .map(NativeShaderBufferParameter::align)
            .max()
            .unwrap_or(1)
    }

    fn total_buffer_size(&self) -> usize {
        let mut size = 0usize;

        for param in &self.buffer_params {
            size = size.next_multiple_of(param.align());
            size += param.size();
        }

        size
    }

    fn buffer_offset_size(
        buffer_params: &[NativeShaderBufferParameter],
        buffer_idx: usize,
    ) -> (usize, usize) {
        assert!(buffer_idx < buffer_params.len());

        let mut offset: usize = 0;

        for param in buffer_params.iter().take(buffer_idx) {
            offset = offset.next_multiple_of(param.align());
            offset += param.size();
        }

        (offset, buffer_params[buffer_idx].size())
    }

    pub(crate) fn set_parameter(
        &mut self,
        param: &str,
        value: MaterialParameter,
    ) -> Result<(), SetParamErr> {
        profiling::function_scope!();

        let Some(param_index) = self.param_indices.get(param).copied() else {
            return Err(SetParamErr::UnknownParameter);
        };

        match param_index {
            ParamIndex::Buffer(idx) => {
                let idx = idx as usize;

                let conversion_ok = self.buffer_params[idx].set_from(value);

                if !conversion_ok {
                    return Err(SetParamErr::InvalidConversion("<TODO NAME>", "<TODO NAME>"));
                }

                if let Some(buffer) = self.native.as_ref().map(|n| &n.0) {
                    let param_bytes = self.buffer_params[idx].bytes();

                    // We update the subset of the GPU buffer where our parameter lies
                    let (offset, size) = Self::buffer_offset_size(&self.buffer_params, idx);

                    // TODO: Staging belt?
                    let mut queue_write_view = GFX_QUEUE
                        .write_buffer_with(
                            buffer,
                            offset as u64,
                            NonZero::new(size as u64).unwrap(),
                        )
                        .expect("Failed to prepare buffer for write");

                    queue_write_view[..param_bytes.len()].copy_from_slice(param_bytes);
                }

                Ok(())
            }
            ParamIndex::Opaque(idx) => {
                let native_param = &mut self.opaque_params[idx as usize];

                let conversion_ok = native_param.set_from(value);

                if !conversion_ok {
                    return Err(SetParamErr::InvalidConversion("<TODO NAME>", "<TODO NAME>"));
                }

                // Rebinding opaque parameters requires recreating the whole bind group
                self.native = None;

                Ok(())
            }
        }
    }

    pub(crate) fn update_bind_group(&mut self) {
        if self.native.is_some() {
            return;
        }

        profiling::function_scope!(self.bind_group_name.as_str());

        let total_size = self
            .total_buffer_size()
            .max(1)
            .next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT as usize);

        let buffer = GFX_DEVICE.create_buffer(&wgpu::wgt::BufferDescriptor {
            label: Some(format!("{} buffer", self.bind_group_name).as_str()),
            size: total_size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: true,
        });

        let mut buf_slice = buffer.get_mapped_range_mut(..);

        let mut offset: usize = 0;

        for param in &self.buffer_params {
            // Align the parameter to the minimum alignment of its type
            offset = offset.next_multiple_of(param.align());

            // Copy the bytes into the buffer
            let bytes = param.bytes();

            buf_slice[offset..(offset + bytes.len())].copy_from_slice(bytes);

            // Increment the offset by the size of the type.
            offset += bytes.len();
        }

        buffer.unmap();

        let mut entries: Vec<wgpu::BindGroupEntry> =
            Vec::with_capacity(1 + self.opaque_params.len());

        entries.push(wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &buffer,
                offset: 0,
                size: Some(NonZero::new(total_size as u64).unwrap()),
            }),
        });

        for (i, opaque_param) in self.opaque_params.iter().enumerate() {
            let entry = wgpu::BindGroupEntry {
                binding: (i + 1) as u32, // Binding 0 is the buffer binding
                resource: opaque_param.to_binding_resource(),
            };

            entries.push(entry);
        }

        let bind_group = GFX_DEVICE.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&self.bind_group_name),
            layout: &self.layout,
            entries: &entries,
        });

        self.native = Some((buffer, bind_group));
    }

    #[inline]
    pub(crate) fn get_bind_group(&self) -> &wgpu::BindGroup {
        self.native
            .as_ref()
            .map(|native| &native.1)
            .expect("Bind group was dirty or not yet created")
    }
}
