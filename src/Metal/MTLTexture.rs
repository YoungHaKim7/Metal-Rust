//! Safe Metal texture resources and descriptors.

use super::{CompletionToken, PixelFormat, Region, ResourceOptions, StorageMode, TextureUsage};
use crate::Error;

/// A validated descriptor for a two-dimensional Metal texture.
pub struct TextureDescriptor {
    pub(crate) inner: metal_rust_ffi::TextureDescriptor,
}

impl TextureDescriptor {
    /// Creates a 2D descriptor with validated dimensions and allocation hints.
    pub fn new_2d(
        pixel_format: PixelFormat,
        width: usize,
        height: usize,
        storage: ResourceOptions,
        usage: TextureUsage,
    ) -> Result<Self, Error> {
        metal_rust_ffi::TextureDescriptor::new_2d(pixel_format, width, height, storage, usage)
            .map(|inner| Self { inner })
            .map_err(Error::from_ffi)
    }

    /// Sets the number of mip levels.
    pub fn set_mipmap_level_count(&self, count: usize) -> Result<(), Error> {
        self.inner
            .set_mipmap_level_count(count)
            .map_err(Error::from_ffi)
    }
}

/// An owned Metal texture.
#[derive(Clone)]
pub struct Texture {
    pub(crate) inner: metal_rust_ffi::Texture,
}

impl Texture {
    pub(crate) const fn from_ffi(inner: metal_rust_ffi::Texture) -> Self {
        Self { inner }
    }

    /// Returns the texture width.
    #[must_use]
    pub fn width(&self) -> usize {
        self.inner.width()
    }

    /// Returns the texture height.
    #[must_use]
    pub fn height(&self) -> usize {
        self.inner.height()
    }

    /// Returns the texture storage mode.
    #[must_use]
    pub fn storage_mode(&self) -> StorageMode {
        self.inner.storage_mode()
    }

    /// Creates a compatible-format texture view.
    pub fn view_with_pixel_format(&self, pixel_format: PixelFormat) -> Result<Self, Error> {
        self.inner
            .view_with_pixel_format(pixel_format)
            .map(Self::from_ffi)
            .map_err(Error::from_ffi)
    }

    /// Reads a checked 2D region into an owned byte vector after completion.
    pub fn read_region(
        &self,
        region: Region,
        bytes_per_row: usize,
        _completion: &CompletionToken<'_>,
    ) -> Result<Vec<u8>, Error> {
        self.inner
            .read_region(region, bytes_per_row)
            .map_err(Error::from_ffi)
    }
}
