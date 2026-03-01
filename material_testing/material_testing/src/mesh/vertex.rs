use core::any::Any;
use core::num::NonZero;

use crate::ShaderVertexAttributeType;
use crate::types::{GVec2, GVec3};

#[derive(Debug)]
pub(crate) struct VertexBuffer {
    pub(crate) attribute: ShaderVertexAttributeType,
    pub(crate) count: NonZero<u64>,
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) cpu_buffer: Option<Vec<u8>>,
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub(crate) enum NewVertexBufferErr {
    #[display("Cannot create an empty vertex buffer")]
    Zero,

    #[display(
        "Date type {} is not compatible with vertex attribute {}",
        ty,
        attribute
    )]
    IncompatibleDataType {
        ty: &'static str,
        attribute: ShaderVertexAttributeType,
    },
}

impl VertexBuffer {
    pub fn new<T: VertexDataType>(
        data: &[T],
        attribute: ShaderVertexAttributeType,
        device: &wgpu::Device,
        keep_on_cpu: bool,
    ) -> Result<Self, NewVertexBufferErr> {
        profiling::function_scope!();

        if !T::is_compatible_with(attribute) {
            return Err(NewVertexBufferErr::IncompatibleDataType {
                ty: core::any::type_name::<T>(),
                attribute,
            });
        }

        let count: NonZero<u64> =
            NonZero::new(data.len() as u64).ok_or(NewVertexBufferErr::Zero)?;

        let vertex_stride = const {
            assert!((size_of::<T>() as u64).is_multiple_of(wgpu::VERTEX_ALIGNMENT));

            size_of::<T>()
        };

        let buffer_size = (vertex_stride * data.len()) as u64;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: true,
        });

        let bytes = T::as_bytes(data);

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
        })
    }
}

pub trait VertexDataType: Sized + Any {
    fn as_bytes(this: &[Self]) -> &[u8];
    fn is_compatible_with(attribute: ShaderVertexAttributeType) -> bool;
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
        }
    }
}
