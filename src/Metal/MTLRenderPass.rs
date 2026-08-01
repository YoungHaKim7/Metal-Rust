//! Safe render and compute pass descriptors.

use super::{ClearColor, Texture};
use crate::Error;

/// A render-pass descriptor with checked color-attachment indices.
pub struct RenderPassDescriptor {
    pub(crate) inner: metal_rust_ffi::RenderPassDescriptor,
}

impl RenderPassDescriptor {
    /// Creates an empty descriptor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: metal_rust_ffi::RenderPassDescriptor::new(),
        }
    }

    /// Sets a color attachment's texture and clear color.
    pub fn set_color_attachment(
        &self,
        index: usize,
        texture: &Texture,
        clear_color: ClearColor,
    ) -> Result<(), Error> {
        self.inner
            .set_color_attachment(index, &texture.inner, clear_color)
            .map_err(Error::from_ffi)
    }
}

impl Default for RenderPassDescriptor {
    fn default() -> Self {
        Self::new()
    }
}

/// A compute-pass descriptor.
pub struct ComputePassDescriptor {
    pub(crate) inner: metal_rust_ffi::ComputePassDescriptor,
}

impl ComputePassDescriptor {
    /// Creates a default compute-pass descriptor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: metal_rust_ffi::ComputePassDescriptor::new(),
        }
    }
}

impl Default for ComputePassDescriptor {
    fn default() -> Self {
        Self::new()
    }
}
