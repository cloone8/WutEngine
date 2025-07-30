use core::num::NonZero;

use wgpu::wgt::BufferDescriptor;

use crate::GRAPHICS_MANAGER;

pub mod cache;

pub fn create_buffer_uninit(
    size: NonZero<usize>,
    usage: wgpu::BufferUsages,
    name: Option<impl AsRef<str>>,
) -> GpuBuffer {
    let buf = GRAPHICS_MANAGER.device.create_buffer(&BufferDescriptor {
        label: name.as_ref().map(AsRef::as_ref),
        size: size.get() as u64,
        usage,
        mapped_at_creation: false,
    });

    GpuBuffer { buf }
}

pub fn create_buffer_with_data<T>(
    data: &[T],
    usage: wgpu::BufferUsages,
    name: Option<impl AsRef<str>>,
) -> GpuBuffer
where
    T: bytemuck::Pod,
{
    let size = size_of_val(data);

    assert_ne!(0, size, "Cannot create buffer with zero-sized data");

    let buf = GRAPHICS_MANAGER.device.create_buffer(&BufferDescriptor {
        label: name.as_ref().map(AsRef::as_ref),
        size: size as u64,
        usage,
        mapped_at_creation: true,
    });

    let mut buf_view = buf.get_mapped_range_mut(..);

    buf_view.copy_from_slice(bytemuck::cast_slice(data));

    drop(buf_view);

    buf.unmap();

    GpuBuffer { buf }
}

/// A block of arbitrary GPU memory
#[derive(Debug)]
#[must_use]
pub struct GpuBuffer {
    buf: wgpu::Buffer,
}

impl Drop for GpuBuffer {
    fn drop(&mut self) {
        self.buf.destroy();
    }
}

impl GpuBuffer {
    pub fn write_data<T: bytemuck::Pod>(&self, data: &[T]) {
        if self.buf.size() != size_of_val(data) as u64 {
            log::error!(
                "Not writing data to GPU buffer because the size of the data ({}) is not equal to the size of the GPU buffer ({})",
                size_of_val(data),
                self.buf.size()
            );
            return;
        }

        let Some(mut view) = GRAPHICS_MANAGER.queue.write_buffer_with(
            &self.buf,
            0,
            NonZero::new(self.buf.size()).unwrap(),
        ) else {
            log::error!("Could not write to GPU buffer because no view could be created");
            return;
        };

        view.copy_from_slice(bytemuck::cast_slice(data));
    }
}
