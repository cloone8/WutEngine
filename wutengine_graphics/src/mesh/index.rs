use wutengine_asset::assets::mesh::MeshTopology;

/// A raw Mesh index buffer
#[derive(Debug)]
pub struct IndexBuffer {
    /// The topology this buffer contains
    pub(crate) topology: MeshTopology,

    /// The format of each index
    pub(crate) format: IndexFormat,

    /// The amount of indices
    pub(crate) count: usize,

    /// The raw GPU buffer
    pub(crate) buffer: wgpu::Buffer,

    /// The CPU-side stored data
    #[expect(unused, reason = "CPU side mesh modification will be added later")]
    pub(crate) cpu_buffer: Option<Vec<u8>>,
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

impl IndexBuffer {
    /// Creates a new index buffer with the given data and topology
    pub fn new<T: IndexDatatype>(
        data: &[T],
        topology: MeshTopology,
        device: &wgpu::Device,
        keep_on_cpu: bool,
    ) -> Result<Self, NewIndexBufferErr> {
        profiling::function_scope!();

        log::trace!(
            "Creating new index buffer for topology {topology} with {} elements",
            data.len()
        );

        if data.is_empty() {
            return Err(NewIndexBufferErr::Zero);
        }

        if !data.len().is_multiple_of(topology.indices_per_primitive()) {
            return Err(NewIndexBufferErr::NotEnoughIndices {
                count: data.len(),
                topology,
            });
        }

        let data_format = T::FORMAT;
        let data_bytes = <T as IndexDatatype>::as_bytes(data);

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index buffer"),
            size: data_bytes.len() as u64,
            usage: wgpu::BufferUsages::INDEX,
            mapped_at_creation: true,
        });

        let mut buffer_view = buffer.get_mapped_range_mut(..);

        buffer_view.copy_from_slice(data_bytes);

        drop(buffer_view);
        buffer.unmap();

        Ok(Self {
            topology,
            format: data_format,
            count: data.len(),
            buffer,
            cpu_buffer: if keep_on_cpu {
                Some(data_bytes.to_vec())
            } else {
                None
            },
        })
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
    pub fn len(&self) -> usize {
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
}
