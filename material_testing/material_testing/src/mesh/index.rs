use super::MeshTopology;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::Display, derive_more::Error)]
pub(crate) enum NewIndexBufferErr {
    #[display("Cannot create an empty index buffer")]
    Zero,

    #[display("Index count not divisible by {} (required by topology {}): {}", topology.indices_per_primitive(), topology, count)]
    NotEnoughIndices {
        count: usize,
        topology: MeshTopology,
    },
}

#[derive(Debug)]
pub(crate) struct IndexBuffer {
    pub(crate) topology: MeshTopology,
    pub(crate) format: IndexFormat,
    pub(crate) count: usize,
    pub(crate) buffer: wgpu::Buffer,
}

impl IndexBuffer {
    pub fn new<T: IndexDatatype>(
        data: &[T],
        topology: MeshTopology,
        device: &wgpu::Device,
    ) -> Result<Self, NewIndexBufferErr> {
        profiling::function_scope!();

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
        })
    }
}

pub(crate) trait IndexDatatype: Sized {
    const FORMAT: IndexFormat;
    fn as_bytes(this: &[Self]) -> &[u8];
}

impl IndexDatatype for u16 {
    const FORMAT: IndexFormat = IndexFormat::U16;

    #[inline]
    fn as_bytes(this: &[Self]) -> &[u8]
    where
        Self: std::marker::Sized,
    {
        bytemuck::must_cast_slice(this)
    }
}

impl IndexDatatype for u32 {
    const FORMAT: IndexFormat = IndexFormat::U32;

    #[inline]
    fn as_bytes(this: &[Self]) -> &[u8]
    where
        Self: std::marker::Sized,
    {
        bytemuck::must_cast_slice(this)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum IndexFormat {
    U16,
    U32,
}

impl IndexFormat {
    pub(crate) const fn to_wgpu(self) -> wgpu::IndexFormat {
        match self {
            Self::U16 => wgpu::IndexFormat::Uint16,
            Self::U32 => wgpu::IndexFormat::Uint32,
        }
    }
}
