use core::any::Any;
use core::num::NonZero;

use wutengine_asset::assets::shader::ShaderVertexAttributeType;

use crate::shader::GVec4;
use crate::shader::{GVec2, GVec3};

/// A raw vertex buffer
#[derive(Debug)]
pub struct VertexBuffer {
    /// For which attribute this buffer contains data
    #[expect(unused, reason = "CPU side mesh modification will be added later")]
    pub(crate) attribute: ShaderVertexAttributeType,

    /// The amount of elements in this buffer
    #[expect(unused, reason = "CPU side mesh modification will be added later")]
    pub(crate) count: NonZero<u64>,

    /// The handle to the GPU buffer
    pub(crate) buffer: wgpu::Buffer,

    /// A copy of the data in the GPU buffer
    #[expect(unused, reason = "CPU side mesh modification will be added later")]
    pub(crate) cpu_buffer: Option<Vec<u8>>,
}

/// An error while creating a new vertex buffer
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

impl VertexBuffer {
    /// Creates a new vertex buffer filled with the given data
    pub fn new<T: VertexDataType>(
        data: &[T],
        attribute: ShaderVertexAttributeType,
        device: &wgpu::Device,
        keep_on_cpu: bool,
    ) -> Result<Self, NewVertexBufferErr> {
        profiling::function_scope!();

        log::trace!(
            "Creating new vertex buffer for attribute {attribute} with {} elements",
            data.len()
        );

        if !T::is_compatible_with(attribute) {
            return Err(NewVertexBufferErr::IncompatibleDataType {
                ty: core::any::type_name::<T>(),
                attribute,
            });
        }

        let count: NonZero<u64> =
            NonZero::new(data.len() as u64).ok_or(NewVertexBufferErr::Zero)?;

        let vertex_stride = const {
            assert!(
                (size_of::<T>() as u64).is_multiple_of(wgpu::VERTEX_ALIGNMENT),
                "Datatype is not sized to a multiple of the vertex alignment",
            );

            size_of::<T>()
        };

        let buffer_size = (vertex_stride * data.len()) as u64;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Vertex buffer {}", attribute)),
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

    /// Returns a reference to the raw [wgpu::Buffer]
    #[inline(always)]
    pub fn raw(&self) -> &wgpu::Buffer {
        &self.buffer
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
