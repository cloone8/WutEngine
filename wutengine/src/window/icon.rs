use std::path::PathBuf;

use image::GenericImageView;

/// A window icon. The icon dimensions must be equal on both sides, and a power of two
#[derive(Debug, Clone)]
pub enum Icon {
    /// The icon will be set from the given file
    File(PathBuf),

    /// The icon will be set from the given bytes
    /// The bytes will be decoded as if they came straight from an image file
    Bytes(Vec<u8>),
}

impl Icon {
    /// Converts this user-provided icon into a native icon. If this fails, logs
    /// the error and returns [None]
    pub(crate) fn into_native_icon(self) -> Option<winit::window::Icon> {
        let image = match self {
            Self::File(path) => match image::open(path) {
                Ok(img) => img,
                Err(e) => {
                    log::error!("Failed to read icon as an image due to error: {e}");
                    return None;
                }
            },
            Self::Bytes(data) => match image::load_from_memory(&data) {
                Ok(img) => img,
                Err(e) => {
                    log::error!("Failed to read icon bytes as an image due to error: {e}");
                    return None;
                }
            },
        };

        let dims = image.dimensions();

        if dims.0 != dims.1 || !dims.0.is_power_of_two() {
            log::error!(
                "Invalid icon dimensions: ({}, {}). Width/height must be equal and a power of two.",
                dims.0,
                dims.1
            );
            return None;
        }

        let as_rgba = image.into_rgba8().into_raw();

        match winit::window::Icon::from_rgba(as_rgba, dims.0, dims.0) {
            Ok(icon) => Some(icon),
            Err(e) => {
                log::error!("Failed to convert image to a native icon: {e}");
                None
            }
        }
    }
}
