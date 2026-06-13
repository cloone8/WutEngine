use core::any::Any;
use core::num::NonZero;

use wutengine_asset::assets::shader::ShaderVertexAttributeType;

use crate::shader::GVec4;
use crate::shader::{GVec2, GVec3};

/// A raw vertex buffer
#[derive(Debug)]
pub struct VertexBuffer {
    /// For which attribute this buffer contains data
    attribute: ShaderVertexAttributeType,

    /// The amount of elements in this buffer
    count: NonZero<u64>,

    /// Whether the vertex buffer can be modified dynamically
    dynamic: bool,

    /// The handle to the GPU buffer
    buffer: wgpu::Buffer,

    /// A copy of the data in the GPU buffer
    #[expect(unused, reason = "CPU side mesh modification will be added later")]
    cpu_buffer: Option<Vec<u8>>,
}

/// An error while creating a new [VertexBuffer]
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum NewVertexBufferErr {
    #[display("Cannot create an empty vertex buffer")]
    /// Vertex buffer cannot be empty
    Zero,

    #[display(
        "Datatype {} is not compatible with vertex attribute {}",
        ty,
        attribute
    )]
    /// The data type is not compatible with the requested vertex attribute
    IncompatibleDataType {
        /// The given datatype
        ty: &'static str,

        /// The requested attribute
        attribute: ShaderVertexAttributeType,
    },
}

/// An error while updating a [VertexBuffer]
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum UpdateVertexBufferErr {
    #[display("Cannot update a non-dynamic vertex buffer")]
    /// Vertex buffer cannot be empty
    NotDynamic,

    /// Out of bounds
    #[display(
        "Update would go out of buffer bounds. Tried to write elements {write_start}..{write_end}, buffer length: {buf_len}"
    )]
    OutOfRange {
        /// Start index of attempted write
        write_start: u64,

        /// Exclusive end index of attempted write
        write_end: u64,

        /// Length of buffer
        buf_len: u64,
    },

    #[display(
        "Datatype {} is not compatible with vertex attribute {}",
        ty,
        attribute
    )]
    /// The data type is not compatible with the requested vertex attribute
    IncompatibleDataType {
        /// The given datatype
        ty: &'static str,

        /// The requested attribute
        attribute: ShaderVertexAttributeType,
    },
}

impl VertexBuffer {
    /// Creates a new vertex buffer filled with the given data
    pub fn new<T: VertexDataType>(
        data: &[T],
        attribute: ShaderVertexAttributeType,
        device: &wgpu::Device,
        keep_on_cpu: bool,
        dynamic: bool,
    ) -> Result<Self, NewVertexBufferErr> {
        profiling::function_scope!();

        let count: NonZero<u64> =
            NonZero::new(data.len() as u64).ok_or(NewVertexBufferErr::Zero)?;

        let buffer = Self::alloc_buffer::<T>(attribute, count, dynamic, device)?;

        let bytes = T::as_bytes(data);

        assert_eq!(
            buffer.size(),
            bytes.len() as u64,
            "Unexpected number of bytes"
        );

        let mut buffer_view = buffer.get_mapped_range_mut(..);

        buffer_view.copy_from_slice(bytes);

        drop(buffer_view);
        buffer.unmap();

        let cpu_buffer = if keep_on_cpu {
            Some(bytes.to_vec())
        } else {
            None
        };

        Ok(Self {
            attribute,
            count,
            buffer,
            cpu_buffer,
            dynamic,
        })
    }

    /// Creates a new vertex buffer and write data directly into it
    pub fn new_direct<T: VertexDataType>(
        count: NonZero<u64>,
        attribute: ShaderVertexAttributeType,
        device: &wgpu::Device,
        keep_on_cpu: bool,
        dynamic: bool,
        write_func: impl FnOnce(&mut wgpu::WriteOnly<'_, [u8]>),
    ) -> Result<Self, NewVertexBufferErr> {
        profiling::function_scope!();

        let buffer = Self::alloc_buffer::<T>(attribute, count, dynamic, device)?;

        let mut buf_view = buffer.get_mapped_range_mut(..);
        let mut buf_view_slice = buf_view.slice(..);

        write_func(&mut buf_view_slice);

        drop(buf_view);

        let cpu_buffer = if keep_on_cpu {
            Some(buffer.get_mapped_range(..).to_vec())
        } else {
            None
        };

        buffer.unmap();

        Ok(Self {
            attribute,
            count,
            buffer,
            cpu_buffer,
            dynamic,
        })
    }

    fn alloc_buffer<T: VertexDataType>(
        attribute: ShaderVertexAttributeType,
        count: NonZero<u64>,
        dynamic: bool,
        device: &wgpu::Device,
    ) -> Result<wgpu::Buffer, NewVertexBufferErr> {
        profiling::function_scope!();

        log::trace!("Creating new vertex buffer for attribute {attribute} with {count} elements");

        if !T::is_compatible_with(attribute) {
            return Err(NewVertexBufferErr::IncompatibleDataType {
                ty: core::any::type_name::<T>(),
                attribute,
            });
        }

        let vertex_stride = const {
            assert!(
                (size_of::<T>() as u64).is_multiple_of(wgpu::VERTEX_ALIGNMENT),
                "Datatype is not sized to a multiple of the vertex alignment",
            );

            size_of::<T>()
        };

        let buffer_size = (vertex_stride as u64) * count.get();

        Ok(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Vertex buffer {}", attribute)),
            size: buffer_size,
            usage: wgpu::BufferUsages::VERTEX
                | (if dynamic {
                    wgpu::BufferUsages::COPY_DST
                } else {
                    wgpu::BufferUsages::empty()
                }),
            mapped_at_creation: true,
        }))
    }

    /// Updates an existing buffer with new data. Buffer must have been created with `dynamic` set to `true`
    pub fn update<T: VertexDataType>(
        &mut self,
        offset: u64,
        data: &[T],
    ) -> Result<(), Box<UpdateVertexBufferErr>> {
        profiling::function_scope!();

        if !self.dynamic {
            return Err(Box::new(UpdateVertexBufferErr::NotDynamic));
        }

        if !T::is_compatible_with(self.attribute) {
            return Err(Box::new(UpdateVertexBufferErr::IncompatibleDataType {
                ty: core::any::type_name::<T>(),
                attribute: self.attribute,
            }));
        }

        if data.is_empty() {
            return Ok(());
        }

        if (offset + data.len() as u64) > self.count.get() {
            return Err(Box::new(UpdateVertexBufferErr::OutOfRange {
                write_start: offset,
                write_end: offset + data.len() as u64,
                buf_len: self.count.get(),
            }));
        }

        let bytes = T::as_bytes(data);

        {
            profiling::scope!("Write buffer");
            let mut buf_view = crate::queue()
                .write_buffer_with(
                    &self.buffer,
                    offset,
                    NonZero::new(bytes.len() as u64).unwrap(),
                )
                .expect("Should have returned a view");

            buf_view.copy_from_slice(bytes);

            drop(buf_view);
        }

        Ok(())
    }

    /// Updates an existing buffer with new data by writing directly into it. Buffer must have been created with `dynamic` set to `true`
    pub fn update_direct<T: VertexDataType>(
        &mut self,
        offset: u64,
        count: u64,
        update_func: impl FnOnce(&mut wgpu::WriteOnly<'_, [u8]>),
    ) -> Result<(), Box<UpdateVertexBufferErr>> {
        profiling::function_scope!();

        if !self.dynamic {
            return Err(Box::new(UpdateVertexBufferErr::NotDynamic));
        }

        if (offset + count) > self.count.get() {
            return Err(Box::new(UpdateVertexBufferErr::OutOfRange {
                write_start: offset,
                write_end: offset + count,
                buf_len: self.count.get(),
            }));
        }

        if count == 0 {
            return Ok(());
        }

        let mut buf_view = crate::queue()
            .write_buffer_with(
                &self.buffer,
                offset,
                NonZero::new(count * size_of::<T>() as u64).unwrap(),
            )
            .expect("Should have returned a view");
        let mut buf_view_slice = buf_view.slice(..);

        update_func(&mut buf_view_slice);

        Ok(())
    }

    /// Returns a reference to the raw [wgpu::Buffer]
    #[inline(always)]
    pub fn raw(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Returns the amount of elements in this buffer
    #[inline(always)]
    pub fn len(&self) -> NonZero<u64> {
        self.count
    }
}

/// A type that is usable as the content of a [VertexBuffer]
pub trait VertexDataType: Sized + Any {
    /// Casts the given slice to a byte slice
    fn as_bytes(this: &[Self]) -> &[u8];

    /// Returns whether this type is usable as the given vertex attribute
    fn is_compatible_with(attribute: ShaderVertexAttributeType) -> bool;
}

impl VertexDataType for GVec4<f32> {
    #[inline]
    fn as_bytes(this: &[Self]) -> &[u8] {
        bytemuck::must_cast_slice(this)
    }

    fn is_compatible_with(attribute: ShaderVertexAttributeType) -> bool {
        match attribute {
            ShaderVertexAttributeType::Position => false,
            ShaderVertexAttributeType::Uv { .. } => false,
            ShaderVertexAttributeType::Color => true,
        }
    }
}

impl VertexDataType for GVec3<f32> {
    #[inline]
    fn as_bytes(this: &[Self]) -> &[u8] {
        bytemuck::must_cast_slice(this)
    }

    fn is_compatible_with(attribute: ShaderVertexAttributeType) -> bool {
        match attribute {
            ShaderVertexAttributeType::Position => true,
            ShaderVertexAttributeType::Uv { .. } => false,
            ShaderVertexAttributeType::Color => false,
        }
    }
}

impl VertexDataType for GVec2<f32> {
    #[inline]
    fn as_bytes(this: &[Self]) -> &[u8] {
        bytemuck::must_cast_slice(this)
    }

    fn is_compatible_with(attribute: ShaderVertexAttributeType) -> bool {
        match attribute {
            ShaderVertexAttributeType::Position => false,
            ShaderVertexAttributeType::Uv { .. } => true,
            ShaderVertexAttributeType::Color => false,
        }
    }
}

/// Returns the amount of bytes used by a given shader vertex attribute
pub const fn attr_bytes(attr: ShaderVertexAttributeType) -> usize {
    match attr {
        ShaderVertexAttributeType::Position => size_of::<GVec3<f32>>(),
        ShaderVertexAttributeType::Uv { .. } => size_of::<GVec2<f32>>(),
        ShaderVertexAttributeType::Color => size_of::<GVec4<f32>>(),
    }
}
