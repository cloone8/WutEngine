use core::ops::Deref;
use core::sync::atomic::{AtomicU16, Ordering, fence};

use image::imageops::FilterType::Gaussian;
use serde::{Deserialize, Serialize};
use wutengine_asset::serializers::image::dynamic_image::serialize;

/// Every time we lose the GPU
static GPU_DEVICE_GENERATION: AtomicU16 = AtomicU16::new(0);

pub(crate) fn increment_device_generation() {
    GPU_DEVICE_GENERATION.fetch_add(1, Ordering::Release);
}

#[derive(Debug, Clone)]
pub struct GpuResource<T> {
    inner: Option<T>,

    device_generation: u16,
}

impl<T> Default for GpuResource<T> {
    fn default() -> Self {
        Self {
            inner: None,
            device_generation: GPU_DEVICE_GENERATION.load(Ordering::Acquire),
        }
    }
}

impl<T> GpuResource<T> {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn set(&mut self, val: T) {
        self.inner = Some(val);
        self.device_generation = GPU_DEVICE_GENERATION.load(Ordering::Acquire);
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.inner = None;
    }

    #[inline(always)]
    pub fn get(&self) -> Option<&T> {
        self.inner
            .as_ref()
            .filter(|_| self.device_generation == GPU_DEVICE_GENERATION.load(Ordering::Acquire))
    }

    #[inline(always)]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.inner
            .as_mut()
            .filter(|_| self.device_generation == GPU_DEVICE_GENERATION.load(Ordering::Acquire))
    }

    #[inline(always)]
    pub fn is_loaded(&self) -> bool {
        self.inner.is_some()
            && self.device_generation == GPU_DEVICE_GENERATION.load(Ordering::Acquire)
    }
}
