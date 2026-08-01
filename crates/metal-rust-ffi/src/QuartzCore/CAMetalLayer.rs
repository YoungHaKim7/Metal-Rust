//! Audited Core Animation and Metal drawable boundary.

use crate::foundation::Error;
use crate::metal::{Device, PixelFormat, Texture};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_quartz_core::{CAMetalDrawable, CAMetalLayer};

/// An owned `CAMetalLayer`.
pub struct Layer {
    pub(super) inner: Retained<CAMetalLayer>,
}

impl Layer {
    /// Creates a new Core Animation Metal layer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: CAMetalLayer::layer(),
        }
    }

    /// Creates a layer associated with a specific Metal device.
    #[must_use]
    pub fn with_device(device: &Device) -> Self {
        let layer = Self::new();
        layer.inner.setDevice(Some(&device.inner));
        layer
    }

    /// Sets the Metal device used for drawable allocation.
    pub fn set_device(&self, device: Option<&Device>) {
        self.inner.setDevice(device.map(|device| &*device.inner));
    }

    /// Returns the layer's current device, if one has been selected.
    #[must_use]
    pub fn device(&self) -> Option<Device> {
        self.inner.device().map(Device::from_inner)
    }

    /// Sets the drawable pixel format.
    pub fn set_pixel_format(&self, pixel_format: PixelFormat) {
        self.inner.setPixelFormat(pixel_format.as_objc());
    }

    /// Returns the next drawable, if the layer can currently provide one.
    pub fn next_drawable(&self) -> Result<Drawable, Error> {
        self.inner
            .nextDrawable()
            .map(Drawable::new)
            .ok_or_else(|| Error::unsupported("Core Animation could not provide a drawable"))
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self::new()
    }
}

/// An owned drawable obtained from a `CAMetalLayer`.
pub struct Drawable {
    pub(crate) inner: Retained<ProtocolObject<dyn CAMetalDrawable>>,
}

impl Drawable {
    fn new(inner: Retained<ProtocolObject<dyn CAMetalDrawable>>) -> Self {
        Self { inner }
    }

    /// Returns the texture backing this drawable.
    #[must_use]
    pub fn texture(&self) -> Texture {
        Texture::new(self.inner.texture())
    }
}
