//! Safe Core Animation Metal layer and drawable wrappers.

use crate::{Device, Error, PixelFormat, Texture};

/// An owned `CAMetalLayer`.
pub struct Layer {
    pub(crate) inner: metal_rust_ffi::Layer,
}

impl Layer {
    /// Creates a new Metal layer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: metal_rust_ffi::Layer::new(),
        }
    }

    /// Creates a layer associated with a device.
    #[must_use]
    pub fn with_device(device: &Device) -> Self {
        Self {
            inner: metal_rust_ffi::Layer::with_device(&device.inner),
        }
    }

    /// Sets the layer's Metal device.
    pub fn set_device(&self, device: Option<&Device>) {
        self.inner.set_device(device.map(|device| &device.inner));
    }

    /// Returns the current device, if configured.
    #[must_use]
    pub fn device(&self) -> Option<Device> {
        self.inner.device().map(Device::from_ffi)
    }

    /// Sets the pixel format used for future drawables.
    pub fn set_pixel_format(&self, pixel_format: PixelFormat) {
        self.inner.set_pixel_format(pixel_format);
    }

    /// Acquires the next drawable.
    pub fn next_drawable(&self) -> Result<Drawable, Error> {
        self.inner
            .next_drawable()
            .map(|inner| Drawable { inner })
            .map_err(Error::from_ffi)
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self::new()
    }
}

/// An owned drawable obtained from a Metal layer.
pub struct Drawable {
    pub(crate) inner: metal_rust_ffi::Drawable,
}

impl Drawable {
    /// Returns the drawable's texture.
    #[must_use]
    pub fn texture(&self) -> Texture {
        Texture::from_ffi(self.inner.texture())
    }
}
