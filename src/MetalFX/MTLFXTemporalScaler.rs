//! Safe MetalFX temporal scaler API.

use crate::{Device, Error, PixelFormat, Texture};

/// A temporal-scaler descriptor.
pub struct TemporalScalerDescriptor {
    pub(crate) inner: metal_rust_ffi::TemporalScalerDescriptor,
}

impl TemporalScalerDescriptor {
    /// Creates a descriptor when MetalFX is available on the current system.
    pub fn new() -> Result<Self, Error> {
        metal_rust_ffi::TemporalScalerDescriptor::new()
            .map(|inner| Self { inner })
            .map_err(Error::from_ffi)
    }

    /// Configures formats and dimensions after validating non-zero sizes.
    #[allow(clippy::too_many_arguments)]
    pub fn configure(
        &self,
        color_format: PixelFormat,
        depth_format: PixelFormat,
        motion_format: PixelFormat,
        output_format: PixelFormat,
        input_width: usize,
        input_height: usize,
        output_width: usize,
        output_height: usize,
    ) -> Result<(), Error> {
        self.inner
            .configure(
                color_format,
                depth_format,
                motion_format,
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

    /// Returns the minimum supported input content scale.
    #[must_use]
    pub fn supported_input_content_min_scale(&self, device: &Device) -> f32 {
        self.inner.supported_input_content_min_scale(&device.inner)
    }

    /// Returns the maximum supported input content scale.
    #[must_use]
    pub fn supported_input_content_max_scale(&self, device: &Device) -> f32 {
        self.inner.supported_input_content_max_scale(&device.inner)
    }

    /// Creates a temporal scaler.
    pub fn new_scaler(&self, device: &Device) -> Result<TemporalScaler, Error> {
        self.inner
            .new_scaler(&device.inner)
            .map(|inner| TemporalScaler { inner })
            .map_err(Error::from_ffi)
    }
}

/// An owned MetalFX temporal scaler.
pub struct TemporalScaler {
    pub(crate) inner: metal_rust_ffi::TemporalScaler,
}

impl TemporalScaler {
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

    /// Returns the required input depth texture usage bitmask.
    #[must_use]
    pub fn depth_texture_usage(&self) -> u64 {
        self.inner.depth_texture_usage()
    }

    /// Returns the required input motion texture usage bitmask.
    #[must_use]
    pub fn motion_texture_usage(&self) -> u64 {
        self.inner.motion_texture_usage()
    }

    /// Sets the input content dimensions after checking they are non-zero.
    pub fn set_input_content_size(&self, width: usize, height: usize) -> Result<(), Error> {
        self.inner
            .set_input_content_size(width, height)
            .map_err(Error::from_ffi)
    }

    /// Assigns the color, depth, motion, and output textures. The output must
    /// use private storage.
    pub fn set_textures(
        &self,
        color: Option<&Texture>,
        depth: Option<&Texture>,
        motion: Option<&Texture>,
        output: Option<&Texture>,
    ) -> Result<(), Error> {
        self.inner
            .set_textures(
                color.map(|texture| &texture.inner),
                depth.map(|texture| &texture.inner),
                motion.map(|texture| &texture.inner),
                output.map(|texture| &texture.inner),
            )
            .map_err(Error::from_ffi)
    }
}
