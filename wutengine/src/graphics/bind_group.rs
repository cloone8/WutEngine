use core::num::NonZero;
use std::collections::HashMap;

use super::material::MaterialParameter;
use super::shader::{
    ShaderBufferParameter, ShaderBufferParameterType, ShaderOpaqueParameter, ShaderParameter,
};

#[derive(Debug, Clone)]
pub(crate) struct BindGroup {
    bind_group_name: String,
    param_indices: HashMap<String, ParamIndex>,
    buffer_params: Vec<ShaderBufferParameter>,
    opaque_params: Vec<ShaderOpaqueParameter>,
    pub(crate) layout: wgpu::BindGroupLayout,
    native: Option<(wgpu::Buffer, wgpu::BindGroup)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ParamIndex {
    Buffer(u16),
    Opaque(u16),
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum SetParamErr {
    #[display("Unkown parameter in bind group: {}", _0)]
    UnknownParameter(#[error(not(source))] String),

    #[display("Cannot convert value of type {} to parameter type {}", from, to)]
    InvalidConversion {
        from: &'static str,
        to: &'static str,
    },
}

impl BindGroup {
    pub fn new<'a>(
        name: String,
        layout: wgpu::BindGroupLayout,
        params: impl IntoIterator<Item = &'a ShaderParameter>,
    ) -> Self {
        let mut param_indices = HashMap::new();
        let mut buffer_params = Vec::new();
        let mut opaque_params = Vec::new();

        for param in params {
            match param {
                ShaderParameter::Buffer { ty, name, .. } => {
                    let index = ParamIndex::Buffer(buffer_params.len() as u16);

                    buffer_params.push(ty.to_default_value());
                    let prev = param_indices.insert(name.clone(), index);

                    assert!(prev.is_none());
                }
                ShaderParameter::Opaque { ty, name, .. } => {
                    let index = ParamIndex::Opaque(opaque_params.len() as u16);

                    opaque_params.push(ty.to_default_value());

                    let prev = param_indices.insert(name.clone(), index);
                    assert!(prev.is_none());
                }
            }
        }

        Self {
            bind_group_name: name,
            param_indices,
            buffer_params,
            opaque_params,
            layout,
            native: None,
        }
    }

    #[inline]
    pub fn total_buffer_align(
        params: impl IntoIterator<Item = impl Into<ShaderBufferParameterType>>,
    ) -> usize {
        params
            .into_iter()
            .map(|p| p.into().align())
            .max()
            .unwrap_or(1)
    }

    #[inline]
    pub fn total_buffer_size(
        params: impl IntoIterator<Item = impl Into<ShaderBufferParameterType>>,
    ) -> usize {
        let mut size = 0usize;

        for param in params {
            let param: ShaderBufferParameterType = param.into();
            size = size.next_multiple_of(param.align());
            size += param.size();
        }

        size
    }

    fn buffer_offset_size(
        buffer_params: &[ShaderBufferParameter],
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
        queue: &wgpu::Queue,
    ) -> Result<(), SetParamErr> {
        let Some(param_index) = self.param_indices.get(param).copied() else {
            return Err(SetParamErr::UnknownParameter(param.to_owned()));
        };

        match param_index {
            ParamIndex::Buffer(idx) => {
                let idx = idx as usize;

                let conversion_ok = self.buffer_params[idx].set_from(value);

                if !conversion_ok {
                    return Err(SetParamErr::InvalidConversion {
                        from: "<TODO NAME>",
                        to: "<TODO NAME>",
                    });
                }

                if let Some(buffer) = self.native.as_ref().map(|n| &n.0) {
                    let param_bytes = self.buffer_params[idx].bytes();

                    // We update the subset of the GPU buffer where our parameter lies
                    let (offset, size) = Self::buffer_offset_size(&self.buffer_params, idx);

                    // TODO: Staging belt?
                    let mut queue_write_view = queue
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
                let conversion_ok = self.opaque_params[idx as usize].set_from(value);

                if !conversion_ok {
                    return Err(SetParamErr::InvalidConversion {
                        from: "<TODO NAME>",
                        to: "<TODO NAME>",
                    });
                }

                // Rebinding opaque parameters requires recreating the whole bind group
                self.native = None;

                Ok(())
            }
        }
    }

    pub(crate) fn update_bind_group(&mut self, device: &wgpu::Device) {
        if self.native.is_some() {
            // Update is not required
            return;
        }

        let total_size = Self::total_buffer_size(&self.buffer_params)
            .max(1)
            .next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT as usize);

        let buffer = device.create_buffer(&wgpu::wgt::BufferDescriptor {
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

        drop(buf_slice);
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
