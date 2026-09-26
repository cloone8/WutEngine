//! Wrapper around a [`wgpu::BindGroup`]

use core::num::NonZero;
use std::collections::HashMap;

use wutengine_assets::assets::shader::Binding;
use wutengine_assets::assets::shader::BindingKind;
use wutengine_assets::assets::shader::BufferSpace;
use wutengine_assets::assets::shader::TextureDimension;
use wutengine_assets::assets::shader::TextureSampleType;

use crate::GFX_DEVICE;
use crate::label;
use crate::sampler::DEFAULT_SAMPLER;
use crate::texture::DEFAULT_TEXTURE;

use super::material::MaterialParameter;
use super::shader::ShaderBufferParameter;
use super::shader::ShaderOpaqueParameter;

/// A shader binding the runtime can't provide a value for
#[derive(Debug, derive_more::Display, derive_more::Error)]
#[display("Binding `{}` ({:?}) is not supported by the runtime yet", _0.name, _0.kind)]
pub struct UnsupportedBindingErr(#[error(not(source))] Box<Binding>);

/// The layout of a [`BindGroup`]: the native layout, and the bindings it was made from. Can only be made from
/// bindings the runtime supports.
#[derive(Debug, Clone)]
pub struct BindGroupLayout {
    /// The native layout
    native: wgpu::BindGroupLayout,

    /// The bindings
    bindings: Vec<Binding>,
}

impl BindGroupLayout {
    /// Creates the layout for the given bindings
    pub fn new(
        label: Option<&str>,
        bindings: impl IntoIterator<Item = Binding>,
    ) -> Result<Self, UnsupportedBindingErr> {
        profiling::function_scope!();

        let bindings: Vec<Binding> = bindings.into_iter().collect();

        let entries = bindings
            .iter()
            .map(|binding| {
                Ok(wgpu::BindGroupLayoutEntry {
                    binding: binding.binding,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu_binding_type(binding)?,
                    count: None,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let native = GFX_DEVICE.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label,
            entries: &entries,
        });

        Ok(Self { native, bindings })
    }

    /// The native layout
    #[inline]
    pub fn native(&self) -> &wgpu::BindGroupLayout {
        &self.native
    }

    /// The bindings in this layout
    #[inline]
    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }
}

/// Returns the [`wgpu::BindingType`] for a binding, if the runtime can provide values for it
fn wgpu_binding_type(binding: &Binding) -> Result<wgpu::BindingType, UnsupportedBindingErr> {
    let unsupported = || UnsupportedBindingErr(Box::new(binding.clone()));

    Ok(match &binding.kind {
        BindingKind::Buffer {
            space: BufferSpace::Uniform,
            size,
            members,
        } => {
            // Every member must be settable, and the buffer can't be empty
            if members.iter().any(|m| {
                m.array_length != 1 || ShaderBufferParameter::zeroed(m.base_type).is_none()
            }) {
                return Err(unsupported());
            }

            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(NonZero::new(u64::from(*size)).ok_or_else(unsupported)?),
            }
        }
        BindingKind::Sampler { comparison: false } => {
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
        }
        BindingKind::Texture {
            dimension: TextureDimension::D2,
            arrayed: false,
            sample_type: TextureSampleType::Float,
            multisampled: false,
        } => wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        _ => return Err(unsupported()),
    })
}

/// A shader bind group. Holds a set of parameters and their GPU side representation.
#[derive(Debug, Clone)]
pub struct BindGroup {
    bind_group_name: String,
    param_indices: HashMap<String, ParamIndex>,

    /// The CPU-side contents of each buffer binding
    buffers: Vec<BufferData>,
    buffer_params: Vec<BufferParam>,
    opaque_params: Vec<OpaqueParam>,

    /// The layout of this bind group
    layout: BindGroupLayout,

    /// The GPU buffers, in the order of `buffers`, and the bind group
    native: Option<(Vec<wgpu::Buffer>, wgpu::BindGroup)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ParamIndex {
    Buffer(u16),
    Opaque(u16),
}

/// The CPU-side contents of a buffer binding
#[derive(Debug, Clone)]
struct BufferData {
    /// The binding index
    binding: u32,

    /// The buffer contents
    bytes: Vec<u8>,
}

/// A parameter stored in a buffer binding
#[derive(Debug, Clone)]
struct BufferParam {
    /// Index into [`BindGroup::buffers`]
    buffer: u16,

    /// Byte offset within the buffer
    offset: usize,

    /// The current value
    value: ShaderBufferParameter,
}

/// A parameter bound directly, like a texture or sampler
#[derive(Debug, Clone)]
struct OpaqueParam {
    /// The binding index
    binding: u32,

    /// The current value
    value: ShaderOpaqueParameter,
}

/// An error while trying to set a [`BindGroup`] parameter
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
    /// Creates a new bind group with the given name and layout. Buffer members are named by their member name,
    /// other bindings by their global variable name.
    pub fn new(name: String, layout: &BindGroupLayout) -> Self {
        profiling::function_scope!();

        let mut param_indices = HashMap::new();
        let mut buffers = Vec::new();
        let mut buffer_params = Vec::new();
        let mut opaque_params = Vec::new();

        let mut insert_name = |name: &str, index| {
            let prev = param_indices.insert(name.to_owned(), index);
            assert!(prev.is_none(), "Duplicate bindgroup parameter name: {name}");
        };

        for binding in &layout.bindings {
            let opaque_value = match &binding.kind {
                BindingKind::Buffer { size, members, .. } => {
                    let buffer = u16::try_from(buffers.len()).unwrap();

                    // Buffer sizes must be a multiple of this for writes
                    let size = (*size as usize)
                        .next_multiple_of(usize::try_from(wgpu::COPY_BUFFER_ALIGNMENT).unwrap());

                    buffers.push(BufferData {
                        binding: binding.binding,
                        bytes: vec![0; size],
                    });

                    for member in members {
                        insert_name(
                            &member.name,
                            ParamIndex::Buffer(u16::try_from(buffer_params.len()).unwrap()),
                        );

                        buffer_params.push(BufferParam {
                            buffer,
                            offset: member.offset as usize,
                            value: ShaderBufferParameter::zeroed(member.base_type)
                                .expect("Checked by BindGroupLayout::new"),
                        });
                    }

                    continue;
                }
                BindingKind::Sampler { .. } => {
                    ShaderOpaqueParameter::Sampler(DEFAULT_SAMPLER.get_wgpu().clone())
                }
                BindingKind::Texture { .. } => {
                    ShaderOpaqueParameter::Texture2D(DEFAULT_TEXTURE.get_view().clone())
                }
            };

            insert_name(
                &binding.name,
                ParamIndex::Opaque(u16::try_from(opaque_params.len()).unwrap()),
            );

            opaque_params.push(OpaqueParam {
                binding: binding.binding,
                value: opaque_value,
            });
        }

        Self {
            bind_group_name: name,
            param_indices,
            buffers,
            buffer_params,
            opaque_params,
            layout: layout.clone(),
            native: None,
        }
    }

    /// Updates the value of the given parameter to the new value.
    /// This might require the bind-group to be recreated later using [`Self::update_bind_group`]
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
                let param = &mut self.buffer_params[idx as usize];

                if !param.value.set_from(&value) {
                    return Err(SetParamErr::InvalidConversion {
                        from: value_type_name,
                        to: param.value.variant_name(),
                    });
                }

                let param_bytes = param.value.bytes();

                self.buffers[param.buffer as usize].bytes
                    [param.offset..(param.offset + param_bytes.len())]
                    .copy_from_slice(param_bytes);

                if let Some((gpu_buffers, _)) = &self.native {
                    // TODO: Staging belt?
                    queue.write_buffer(
                        &gpu_buffers[param.buffer as usize],
                        param.offset as u64,
                        param_bytes,
                    );
                }

                Ok(())
            }
            ParamIndex::Opaque(idx) => {
                let param = &mut self.opaque_params[idx as usize];

                if !param.value.set_from(value) {
                    return Err(SetParamErr::InvalidConversion {
                        from: value_type_name,
                        to: param.value.variant_name(),
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

        let gpu_buffers: Vec<wgpu::Buffer> = self
            .buffers
            .iter()
            .map(|data| {
                let buffer = device.create_buffer(&wgpu::wgt::BufferDescriptor {
                    label: label!("{} buffer {}", self.bind_group_name, data.binding),
                    size: data.bytes.len() as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                    mapped_at_creation: true,
                });

                buffer
                    .get_mapped_range_mut(..)
                    .expect("Invalid buffer range")
                    .slice(..)
                    .copy_from_slice(&data.bytes);

                buffer.unmap();
                buffer
            })
            .collect();

        let entries: Vec<wgpu::BindGroupEntry> = self
            .buffers
            .iter()
            .zip(&gpu_buffers)
            .map(|(data, buffer)| wgpu::BindGroupEntry {
                binding: data.binding,
                resource: buffer.as_entire_binding(),
            })
            .chain(self.opaque_params.iter().map(|param| wgpu::BindGroupEntry {
                binding: param.binding,
                resource: param.value.to_binding_resource(),
            }))
            .collect();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: label!(&self.bind_group_name),
            layout: self.layout.native(),
            entries: &entries,
        });

        self.native = Some((gpu_buffers, bind_group));
    }

    /// Returns the native [`wgpu::BindGroup`]
    #[inline]
    pub fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.native.as_ref().map(|native| &native.1)
    }

    /// Returns the layout of this bind group
    #[inline]
    pub fn layout(&self) -> &BindGroupLayout {
        &self.layout
    }
}

#[cfg(test)]
mod test {
    use wutengine_assets::assets::shader::BufferBaseType;
    use wutengine_assets::assets::shader::BufferMember;

    use super::*;

    fn uniform(base_type: BufferBaseType, array_length: u32) -> Binding {
        Binding {
            name: "params".to_owned(),
            group: 1,
            binding: 0,
            kind: BindingKind::Buffer {
                space: BufferSpace::Uniform,
                size: 16,
                members: vec![BufferMember {
                    name: "value".to_owned(),
                    base_type,
                    array_length,
                    offset: 0,
                    size: 16,
                }],
            },
        }
    }

    /// Tests which bindings the runtime accepts, to prevent a layout for a binding `BindGroup::new` can't fill
    #[test]
    fn supported_bindings() {
        assert!(wgpu_binding_type(&uniform(BufferBaseType::Vec4f, 1)).is_ok());
        assert!(wgpu_binding_type(&uniform(BufferBaseType::Vec4f, 4)).is_err());
        assert!(wgpu_binding_type(&uniform(BufferBaseType::Mat3x3, 1)).is_err());

        let mut storage = uniform(BufferBaseType::Vec4f, 1);
        if let BindingKind::Buffer { space, .. } = &mut storage.kind {
            *space = BufferSpace::Storage { writable: false };
        }
        assert!(wgpu_binding_type(&storage).is_err());

        let comparison = Binding {
            kind: BindingKind::Sampler { comparison: true },
            ..uniform(BufferBaseType::Float, 1)
        };
        assert!(wgpu_binding_type(&comparison).is_err());
    }
}
