//! Wrapper around a [wgpu::BindGroup]

use core::num::NonZero;
use std::collections::HashMap;

use wutengine_assets::assets::shader::ShaderBufferParameterType;
use wutengine_assets::assets::shader::ShaderParameter;

use crate::label;
use crate::shader::shader_buffer_param_default_value;
use crate::shader::shader_opaque_param_default_value;

use super::material::MaterialParameter;
use super::shader::ShaderBufferParameter;
use super::shader::ShaderOpaqueParameter;
use super::shader::shader_buffer_param_align;
use super::shader::shader_buffer_param_size;

/// A shader bind group. Holds a set of parameters and their GPU side representation.
#[derive(Debug, Clone)]
pub struct BindGroup {
    bind_group_name: String,
    param_indices: HashMap<String, ParamIndex>,
    buffer_params: Vec<ShaderBufferParameter>,
    opaque_params: Vec<ShaderOpaqueParameter>,

    /// The native [wgpu] bind group layout for this bind group
    layout: wgpu::BindGroupLayout,

    native: Option<(Option<wgpu::Buffer>, wgpu::BindGroup)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ParamIndex {
    Buffer(u16),
    Opaque(u16),
}

/// An error while trying to set a [BindGroup] parameter
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum SetParamErr {
    /// Unknown parameter
    #[display("Unkown parameter in bind group: {}", _0)]
    UnknownParameter(#[error(not(source))] String),

    /// Invalid type conversion
    #[display(
        "Cannot convert value of type \"{}\" to parameter type \"{}\"",
        from,
        to
    )]
    InvalidConversion {
        /// Source type
        from: &'static str,

        /// Target type
        to: &'static str,
    },
}

impl BindGroup {
    /// Creates a new bind group with the given name, native layout, and parameters
    pub fn new<'a>(
        name: String,
        layout: wgpu::BindGroupLayout,
        params: impl IntoIterator<Item = &'a ShaderParameter>,
    ) -> Self {
        profiling::function_scope!();

        let mut param_indices = HashMap::new();
        let mut buffer_params = Vec::new();
        let mut opaque_params = Vec::new();

        for param in params {
            let (name, index) = match param {
                ShaderParameter::Buffer { ty, name, .. } => {
                    let index = ParamIndex::Buffer(buffer_params.len() as u16);
                    buffer_params.push(shader_buffer_param_default_value(*ty));

                    (name, index)
                }
                ShaderParameter::Opaque { ty, name, .. } => {
                    let index = ParamIndex::Opaque(opaque_params.len() as u16);
                    opaque_params.push(shader_opaque_param_default_value(*ty));

                    (name, index)
                }
            };

            let prev = param_indices.insert(name.clone(), index);

            assert!(prev.is_none(), "Duplicate bindgroup parameter name: {name}");
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

    /// Returns the total size of the GPU buffer needed to hold all given parameters
    #[inline]
    pub(crate) fn total_buffer_size(
        params: impl IntoIterator<Item = impl Into<ShaderBufferParameterType>>,
    ) -> usize {
        let mut size = 0usize;

        for param in params {
            let param: ShaderBufferParameterType = param.into();
            size = size.next_multiple_of(shader_buffer_param_align(param));
            size += shader_buffer_param_size(param);
        }

        size
    }

    fn buffer_offset_size(
        buffer_params: &[ShaderBufferParameter],
        buffer_idx: usize,
    ) -> (usize, usize) {
        assert!(
            buffer_idx < buffer_params.len(),
            "Index out of range: {} (max {})",
            buffer_idx,
            buffer_params.len()
        );

        let mut offset: usize = 0;

        for param in buffer_params.iter().take(buffer_idx) {
            offset = offset.next_multiple_of(param.align());
            offset += param.size();
        }

        (offset, buffer_params[buffer_idx].size())
    }

    /// Updates the value of the given parameter to the new value.
    /// This might require the bind-group to be recreated later using [Self::update_bind_group]
    pub fn set_parameter(
        &mut self,
        param: &str,
        value: MaterialParameter,
        queue: &wgpu::Queue,
    ) -> Result<(), SetParamErr> {
        profiling::function_scope!();

        let Some(param_index) = self.param_indices.get(param).copied() else {
            return Err(SetParamErr::UnknownParameter(param.to_owned()));
        };

        let value_type_name = value.variant_name();

        match param_index {
            ParamIndex::Buffer(idx) => {
                let idx = idx as usize;

                let conversion_ok = self.buffer_params[idx].set_from(value);

                if !conversion_ok {
                    return Err(SetParamErr::InvalidConversion {
                        from: value_type_name,
                        to: self.buffer_params[idx].variant_name(),
                    });
                }

                if let Some(buffer) = self.native.as_ref().map(|n| &n.0) {
                    let buffer = buffer.as_ref().expect("Buffer parameters on bind group, but no buffer was created during bind group updating");

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

                    queue_write_view
                        .slice(..param_bytes.len())
                        .copy_from_slice(param_bytes);
                }

                Ok(())
            }
            ParamIndex::Opaque(idx) => {
                let conversion_ok = self.opaque_params[idx as usize].set_from(value);

                if !conversion_ok {
                    return Err(SetParamErr::InvalidConversion {
                        from: value_type_name,
                        to: self.opaque_params[idx as usize].variant_name(),
                    });
                }

                // Rebinding opaque parameters requires recreating the whole bind group
                self.native = None;

                Ok(())
            }
        }
    }

    /// Updates the GPU side bind-group, if required. Usually only when texture bindings change,
    /// or if the bindgroup itself is new
    pub fn update_bind_group(&mut self, device: &wgpu::Device) {
        if self.native.is_some() {
            // Update is not required
            return;
        }

        profiling::function_scope!();

        let mut entries: Vec<wgpu::BindGroupEntry> =
            Vec::with_capacity(1 + self.opaque_params.len());

        let total_buf_size = Self::total_buffer_size(&self.buffer_params);

        let mut maybe_buffer = None;

        if total_buf_size != 0 {
            let total_buf_size =
                total_buf_size.next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT as usize);

            let buffer = device.create_buffer(&wgpu::wgt::BufferDescriptor {
                label: label!("{} buffer", self.bind_group_name),
                size: total_buf_size as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                mapped_at_creation: true,
            });

            let mut buf_slice = buffer
                .get_mapped_range_mut(..)
                .expect("Invalid buffer range");

            let mut offset: usize = 0;

            for param in &self.buffer_params {
                // Align the parameter to the minimum alignment of its type
                offset = offset.next_multiple_of(param.align());

                // Copy the bytes into the buffer
                let bytes = param.bytes();

                buf_slice
                    .slice(offset..(offset + bytes.len()))
                    .copy_from_slice(bytes);

                // Increment the offset by the size of the type.
                offset += bytes.len();
            }

            drop(buf_slice);
            buffer.unmap();
            maybe_buffer = Some(buffer);

            entries.push(wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: maybe_buffer.as_ref().unwrap(),
                    offset: 0,
                    size: Some(NonZero::new(total_buf_size as u64).unwrap()),
                }),
            });
        }

        for (i, opaque_param) in self.opaque_params.iter().enumerate() {
            let entry = wgpu::BindGroupEntry {
                binding: (i + 1) as u32, // Binding 0 is the buffer binding
                resource: opaque_param.to_binding_resource(),
            };

            entries.push(entry);
        }

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: label!(&self.bind_group_name),
            layout: &self.layout,
            entries: &entries,
        });

        self.native = Some((maybe_buffer, bind_group));
    }

    /// Returns the native [wgpu::BindGroup]
    #[inline]
    pub fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.native.as_ref().map(|native| &native.1)
    }

    /// Returns the native [wgpu::BindGroupLayout]
    #[inline(always)]
    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }
}
