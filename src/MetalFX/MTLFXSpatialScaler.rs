//! Safe MetalFX spatial scaler API.

use crate::{Device, Error, PixelFormat, Texture};

/// A spatial-scaler descriptor.
pub struct SpatialScalerDescriptor {
    pub(crate) inner: metal_rust_ffi::SpatialScalerDescriptor,
}

impl SpatialScalerDescriptor {
    /// Creates a descriptor when MetalFX is available on the current system.
    pub fn new() -> Result<Self, Error> {
        metal_rust_ffi::SpatialScalerDescriptor::new()
            .map(|inner| Self { inner })
            .map_err(Error::from_ffi)
    }

    /// Configures formats and dimensions after validating non-zero sizes.
    pub fn configure(
        &self,
        color_format: PixelFormat,
        output_format: PixelFormat,
        input_width: usize,
        input_height: usize,
        output_width: usize,
        output_height: usize,
    ) -> Result<(), Error> {
        self.inner
            .configure(
                color_format,
                output_format,
                input_width,
                input_height,
                output_width,
                output_height,
            )
            .map_err(Error::from_ffi)
    }

    /// Returns whether the installed MetalFX runtime supports the device.
    #[must_use]
    pub fn supports_device(&self, device: &Device) -> bool {
        self.inner.supports_device(&device.inner)
    }

    /// Creates a spatial scaler.
    pub fn new_scaler(&self, device: &Device) -> Result<SpatialScaler, Error> {
        self.inner
            .new_scaler(&device.inner)
            .map(|inner| SpatialScaler { inner })
            .map_err(Error::from_ffi)
    }
}

/// An owned MetalFX spatial scaler.
pub struct SpatialScaler {
    pub(crate) inner: metal_rust_ffi::SpatialScaler,
}

impl SpatialScaler {
    /// Encodes the scaling operation into a command buffer.
    pub fn encode(&self, command_buffer: &mut crate::CommandBuffer) {
        self.inner.encode(&command_buffer.inner);
    }

    /// Returns the required input texture usage bitmask.
    #[must_use]
    pub fn color_texture_usage(&self) -> u64 {
        self.inner.color_texture_usage()
    }

    /// Returns the required output texture usage bitmask.
    #[must_use]
    pub fn output_texture_usage(&self) -> u64 {
        self.inner.output_texture_usage()
    }

    /// Sets the input content dimensions after checking they are non-zero.
    pub fn set_input_content_size(&self, width: usize, height: usize) -> Result<(), Error> {
        self.inner
            .set_input_content_size(width, height)
            .map_err(Error::from_ffi)
    }

    /// Assigns input and output textures. The output must use private storage.
    pub fn set_textures(
        &self,
        color: Option<&Texture>,
        output: Option<&Texture>,
    ) -> Result<(), Error> {
        self.inner
            .set_textures(
                color.map(|texture| &texture.inner),
                output.map(|texture| &texture.inner),
            )
            .map_err(Error::from_ffi)
    }
}
