use core::num::NonZero;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Mutex, RwLock};

use wgpu::wgt::BufferDescriptor;
use wutengine_util::hash::nohash_hasher::IntMap;

use crate::GRAPHICS_MANAGER;

static CACHED_BUFFER_COUNT: AtomicUsize = AtomicUsize::new(0);

/// The global buffer cache. Contains a pool of buffers of various sizes in order to re-use GPU memory as much as
/// possible
#[derive(Debug)]
pub(crate) struct BufferCache {
    by_size: RwLock<IntMap<u64, SizedBufferCache>>,
}

#[derive(Debug)]
struct SizedBufferCache {
    size: u64,
    used_sink: Sender<wgpu::Buffer>,
    used: Mutex<Receiver<wgpu::Buffer>>,
    fresh: Mutex<Vec<wgpu::Buffer>>,
}

impl SizedBufferCache {
    fn new(size: u64) -> Self {
        let (send, recv) = channel::<wgpu::Buffer>();

        Self {
            size,
            used_sink: send,
            used: Mutex::new(recv),
            fresh: Mutex::new(Vec::new()),
        }
    }

    fn get(&self) -> CachedGpuBuffer {
        let mut fresh_buffers = self.fresh.lock().unwrap();

        match fresh_buffers.pop() {
            Some(buffer) => CachedGpuBuffer {
                return_to_cache: self.used_sink.clone(),
                buffer,
            },
            None => {
                profiling::scope!("Allocate new cached buffer");
                log::debug!("Allocating new cachable GPU buffer of size {}", self.size);

                let new_buffer_num = CACHED_BUFFER_COUNT.fetch_add(1, Ordering::Relaxed);
                let new_buffer = GRAPHICS_MANAGER.device.create_buffer(&BufferDescriptor {
                    label: Some(&format!("Cached buffer {}-{}", self.size, new_buffer_num)),
                    size: self.size,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                    mapped_at_creation: false,
                });

                CachedGpuBuffer {
                    return_to_cache: self.used_sink.clone(),
                    buffer: new_buffer,
                }
            }
        }
    }
}

impl BufferCache {
    /// Creates a new [BufferCache]
    pub(crate) fn new() -> Self {
        Self {
            by_size: RwLock::new(IntMap::default()),
        }
    }
}

/// A cached buffer that automatically returns itself back to the shared buffer cache once dropped
/// Implements [Deref] and [DerefMut] into a [wgpu::Buffer]
#[derive(Debug)]
pub struct CachedGpuBuffer {
    return_to_cache: Sender<wgpu::Buffer>,
    buffer: wgpu::Buffer,
}

impl Deref for CachedGpuBuffer {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl DerefMut for CachedGpuBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

/// Marks all buffers that were used and returned this frame as available again.
/// Should be used once all command queues are submitted
#[profiling::function]
pub fn recycle() {
    let mut buffers = GRAPHICS_MANAGER.buffer_cache.by_size.write().unwrap();

    for buffer_cache in buffers.values_mut() {
        let fresh_buffers = buffer_cache.fresh.get_mut().unwrap();

        let buffer_sink = buffer_cache.used.get_mut().unwrap();

        for used_buffer in buffer_sink.try_iter() {
            fresh_buffers.push(used_buffer);
        }
    }
}

impl Drop for CachedGpuBuffer {
    fn drop(&mut self) {
        if let Err(e) = self.return_to_cache.send(self.buffer.clone()) {
            log::warn!("Failed to return cached buffer back to buffer pool: {e}");
        }
    }
}

/// Returns a new (or recycled) GPU buffer of the given size
#[profiling::function]
pub fn get_buffer(size: usize) -> CachedGpuBuffer {
    let size = size as u64;
    let buffers = GRAPHICS_MANAGER.buffer_cache.by_size.read().unwrap();

    match buffers.get(&size) {
        Some(sized_buffers) => sized_buffers.get(),
        None => {
            drop(buffers);

            let mut buffers = GRAPHICS_MANAGER.buffer_cache.by_size.write().unwrap();

            buffers.insert(size, SizedBufferCache::new(size));
            buffers.get(&size).unwrap().get()
        }
    }
}

/// Returns a cached buffer of the given size, if available. Otherwise, creates a new one
#[profiling::function]
pub fn get_and_write_buffer<T: bytemuck::Pod>(value: &T) -> CachedGpuBuffer {
    const {
        assert!(
            size_of::<T>() > 0,
            "Cannot write zero-sized buffers to the GPU"
        );
    }

    let buffer = get_buffer(size_of::<T>());

    let mut buffer_view = GRAPHICS_MANAGER
        .queue
        .write_buffer_with(&buffer, 0, NonZero::new(size_of::<T>() as u64).unwrap())
        .expect("Failed to write into temporary buffer");

    buffer_view.copy_from_slice(bytemuck::bytes_of(value));

    drop(buffer_view);

    buffer
}
