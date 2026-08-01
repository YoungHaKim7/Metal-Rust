//! Audited texture descriptor and texture resource boundary.

use crate::foundation::Error;
use crate::metal::{PixelFormat, Region, ResourceOptions, StorageMode, TextureType, TextureUsage};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLResource, MTLTexture, MTLTextureDescriptor};

/// A validated descriptor for a two-dimensional Metal texture.
pub struct TextureDescriptor {
    pub(super) inner: Retained<MTLTextureDescriptor>,
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
        if width == 0 || height == 0 {
            return Err(Error::invalid_argument(
                "texture dimensions must be greater than zero",
            ));
        }
        let inner = MTLTextureDescriptor::new();
        inner.setTextureType(TextureType::D2.as_objc());
        inner.setPixelFormat(pixel_format.as_objc());
        // SAFETY: width and height are non-zero usize values and are passed as
        // NSUInteger on the supported Apple targets.
        unsafe {
            inner.setWidth(width);
            inner.setHeight(height);
        }
        inner.setResourceOptions(storage.as_objc());
        inner.setUsage(usage.as_objc());
        Ok(Self { inner })
    }

    /// Sets the number of mip levels after validating the value.
    pub fn set_mipmap_level_count(&self, count: usize) -> Result<(), Error> {
        if count == 0 {
            return Err(Error::invalid_argument(
                "mipmap level count must be greater than zero",
            ));
        }
        // SAFETY: the count is non-zero and fits NSUInteger on supported
        // Apple targets.
        unsafe { self.inner.setMipmapLevelCount(count) };
        Ok(())
    }
}

/// An owned Metal texture resource.
#[derive(Clone)]
pub struct Texture {
    pub(crate) inner: Retained<ProtocolObject<dyn MTLTexture>>,
}

impl Texture {
    pub(crate) const fn new(inner: Retained<ProtocolObject<dyn MTLTexture>>) -> Self {
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
        match self.inner.storageMode().0 {
            1 => StorageMode::Managed,
            2 => StorageMode::Private,
            3 => StorageMode::Memoryless,
            _ => StorageMode::Shared,
        }
    }

    /// Returns a view using a compatible pixel format.
    pub fn view_with_pixel_format(&self, pixel_format: PixelFormat) -> Result<Self, Error> {
        self.inner
            .newTextureViewWithPixelFormat(pixel_format.as_objc())
            .map(Self::new)
            .ok_or_else(|| Error::unsupported("Metal rejected the texture view format"))
    }

    /// Reads a validated 2D region into an owned byte vector.
    pub fn read_region(&self, region: Region, bytes_per_row: usize) -> Result<Vec<u8>, Error> {
        if region.size.width == 0 || region.size.height == 0 || bytes_per_row == 0 {
            return Err(Error::invalid_argument(
                "texture region and row stride must be non-zero",
            ));
        }
        let end_x = region
            .origin
            .x
            .checked_add(region.size.width)
            .ok_or_else(|| Error::invalid_argument("texture region x coordinate overflow"))?;
        let end_y = region
            .origin
            .y
            .checked_add(region.size.height)
            .ok_or_else(|| Error::invalid_argument("texture region y coordinate overflow"))?;
        if end_x > self.width() || end_y > self.height() {
            return Err(Error::invalid_argument("texture region is out of bounds"));
        }
        let byte_count = bytes_per_row
            .checked_mul(region.size.height)
            .ok_or_else(|| Error::invalid_argument("texture read size overflow"))?;
        let mut bytes = vec![0_u8; byte_count];
        let pointer = std::ptr::NonNull::new(bytes.as_mut_ptr().cast())
            .ok_or_else(|| Error::invalid_argument("texture read buffer is empty"))?;
        // SAFETY: the output vector is alive and has byte_count bytes; the
        // region and stride were checked before the Objective-C call.
        unsafe {
            self.inner.getBytes_bytesPerRow_fromRegion_mipmapLevel(
                pointer,
                bytes_per_row,
                region.into(),
                0,
            );
        }
        Ok(bytes)
    }
}
