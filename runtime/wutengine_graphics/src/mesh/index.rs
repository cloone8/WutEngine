use core::num::NonZero;

use alloc::boxed::Box;
use alloc::vec::Vec;
use wutengine_assets::assets::mesh::MeshTopology;

use crate::label;

/// A raw Mesh index buffer
#[derive(Debug)]
pub struct IndexBuffer {
    /// The topology this buffer contains
    topology: MeshTopology,

    /// The format of each index
    format: IndexFormat,

    /// The amount of indices
    count: NonZero<u64>,

    dynamic: bool,

    /// The raw GPU buffer
    buffer: wgpu::Buffer,

    /// The CPU-side stored data
    #[expect(unused, reason = "CPU side mesh modification will be added later")]
    cpu_buffer: Option<Vec<u8>>,
}

/// An error while creating an [IndexBuffer]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::Display, derive_more::Error)]
pub enum NewIndexBufferErr {
    #[display("Cannot create an empty index buffer")]
    /// Cannot create empty index buffer
    Zero,

    #[display("Index count not divisible by {} (required by topology {}): {}", topology.indices_per_primitive(), topology, count)]
    /// Incorrect number of indices was given
    NotEnoughIndices {
        /// The given number of indices
        count: usize,

        /// The actual topology of the buffer
        topology: MeshTopology,
    },
}

/// An error while updating an [IndexBuffer]
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum UpdateIndexBufferErr {
    #[display("Cannot update a non-dynamic index buffer")]
    /// Index buffer cannot be empty
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
}

impl IndexBuffer {
    /// Creates a new index buffer with the given data and topology
    pub fn new<T: IndexDatatype>(
        data: &[T],
        topology: MeshTopology,
        device: &wgpu::Device,
        keep_on_cpu: bool,
        dynamic: bool,
    ) -> Result<Self, NewIndexBufferErr> {
        profiling::function_scope!();

        log::trace!(
            "Creating new index buffer for topology {topology} with {} elements",
            data.len()
        );

        let Some(count) = NonZero::new(data.len() as u64) else {
            return Err(NewIndexBufferErr::Zero);
        };

        let buffer = Self::alloc_buffer::<T>(topology, count, dynamic, device)?;

        let mut buffer_view = buffer
            .get_mapped_range_mut(..)
            .expect("Invalid buffer range");

        let data_bytes = T::as_bytes(data);

        buffer_view.copy_from_slice(data_bytes);

        drop(buffer_view);
        buffer.unmap();

        Ok(Self {
            topology,
            format: T::FORMAT,
            count: NonZero::new(data.len() as u64).unwrap(),
            buffer,
            dynamic,
            cpu_buffer: if keep_on_cpu {
                Some(data_bytes.to_vec())
            } else {
                None
            },
        })
    }

    /// Creates a new index buffer and write data directly into it
    pub fn new_direct<T: IndexDatatype>(
        count: NonZero<u64>,
        topology: MeshTopology,
        device: &wgpu::Device,
        keep_on_cpu: bool,
        dynamic: bool,
        write_func: impl FnOnce(&mut wgpu::WriteOnly<'_, [u8]>),
    ) -> Result<Self, NewIndexBufferErr> {
        profiling::function_scope!();

        let buffer = Self::alloc_buffer::<T>(topology, count, dynamic, device)?;

        let mut buf_view = buffer
            .get_mapped_range_mut(..)
            .expect("Invalid buffer range");
        let mut buf_view_slice = buf_view.slice(..);

        write_func(&mut buf_view_slice);

        drop(buf_view);

        let cpu_buffer = if keep_on_cpu {
            Some(
                buffer
                    .get_mapped_range(..)
                    .expect("Invalid buffer range")
                    .to_vec(),
            )
        } else {
            None
        };

        buffer.unmap();

        Ok(Self {
            topology,
            format: T::FORMAT,
            count,
            buffer,
            cpu_buffer,
            dynamic,
        })
    }

    fn alloc_buffer<T: IndexDatatype>(
        topology: MeshTopology,
        count: NonZero<u64>,
        dynamic: bool,
        device: &wgpu::Device,
    ) -> Result<wgpu::Buffer, NewIndexBufferErr> {
        profiling::function_scope!();

        log::trace!(
            "Creating new index buffer for topology {topology} with {} elements",
            count
        );

        if !count
            .get()
            .is_multiple_of(topology.indices_per_primitive() as u64)
        {
            return Err(NewIndexBufferErr::NotEnoughIndices {
                count: count.get() as usize,
                topology,
            });
        }

        let data_format = T::FORMAT;
        let data_bytes = (data_format.stride() as u64) * count.get();

        Ok(device.create_buffer(&wgpu::BufferDescriptor {
            label: label!("Index buffer"),
            size: data_bytes,
            usage: wgpu::BufferUsages::INDEX
                | (if dynamic {
                    wgpu::BufferUsages::COPY_DST
                } else {
                    wgpu::BufferUsages::empty()
                }),
            mapped_at_creation: true,
        }))
    }

    /// Updates an existing buffer with new data. Buffer must have been created with `dynamic` set to `true`
    pub fn update<T: IndexDatatype>(
        &mut self,
        offset: u64,
        data: &[T],
    ) -> Result<(), Box<UpdateIndexBufferErr>> {
        profiling::function_scope!();

        if !self.dynamic {
            return Err(Box::new(UpdateIndexBufferErr::NotDynamic));
        }

        if data.is_empty() {
            return Ok(());
        }

        if (offset + data.len() as u64) > self.count.get() {
            return Err(Box::new(UpdateIndexBufferErr::OutOfRange {
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
    pub fn update_direct<T: IndexDatatype>(
        &mut self,
        offset: u64,
        count: u64,
        update_func: impl FnOnce(&mut wgpu::WriteOnly<'_, [u8]>),
    ) -> Result<(), Box<UpdateIndexBufferErr>> {
        profiling::function_scope!();

        if !self.dynamic {
            return Err(Box::new(UpdateIndexBufferErr::NotDynamic));
        }

        if (offset + count) > self.count.get() {
            return Err(Box::new(UpdateIndexBufferErr::OutOfRange {
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
                NonZero::new(count * T::FORMAT.stride() as u64).unwrap(),
            )
            .expect("Should have returned a view");
        let mut buf_view_slice = buf_view.slice(..);

        update_func(&mut buf_view_slice);

        Ok(())
    }

    /// Returns the configured [MeshTopology] for this buffer
    #[inline(always)]
    pub fn topology(&self) -> MeshTopology {
        self.topology
    }

    /// Returns a reference to the raw [wgpu::Buffer]
    #[inline(always)]
    pub fn raw(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Returns the format of this buffer
    #[inline(always)]
    pub fn format(&self) -> IndexFormat {
        self.format
    }

    /// Returns the amount of indices in this buffer
    #[inline(always)]
    #[allow(clippy::len_without_is_empty, reason = "Index buffer is never empty")]
    pub fn len(&self) -> NonZero<u64> {
        self.count
    }
}

/// Trait implemented by types that can be used as indices in an [IndexBuffer]
pub trait IndexDatatype: Sized {
    /// The format of this index
    const FORMAT: IndexFormat;

    /// Casts this slice into a byte slice
    fn as_bytes(this: &[Self]) -> &[u8];

    /// Returns the index as a usize for bounds checking
    fn as_usize(&self) -> usize;
}

impl IndexDatatype for u16 {
    const FORMAT: IndexFormat = IndexFormat::U16;

    #[inline]
    fn as_bytes(this: &[Self]) -> &[u8]
    where
        Self: core::marker::Sized,
    {
        bytemuck::must_cast_slice(this)
    }

    #[inline(always)]
    fn as_usize(&self) -> usize {
        *self as usize
    }
}

impl IndexDatatype for u32 {
    const FORMAT: IndexFormat = IndexFormat::U32;

    #[inline]
    fn as_bytes(this: &[Self]) -> &[u8]
    where
        Self: core::marker::Sized,
    {
        bytemuck::must_cast_slice(this)
    }

    #[inline(always)]
    fn as_usize(&self) -> usize {
        *self as usize
    }
}

/// The format of the index buffer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndexFormat {
    /// 16-bit indices
    U16,

    /// 32-bit indices
    U32,
}

impl IndexFormat {
    /// Converts the index format to its [wgpu::IndexFormat] equivalent
    pub const fn to_wgpu(self) -> wgpu::IndexFormat {
        match self {
            Self::U16 => wgpu::IndexFormat::Uint16,
            Self::U32 => wgpu::IndexFormat::Uint32,
        }
    }

    /// The stride in bytes per element of this format
    pub const fn stride(self) -> usize {
        match self {
            Self::U16 => size_of::<u16>(),
            Self::U32 => size_of::<u32>(),
        }
    }
}
