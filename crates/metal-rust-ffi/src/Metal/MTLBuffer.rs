//! Audited buffer allocation and CPU write boundary.

use crate::foundation::Error;
use crate::metal::StorageMode;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_foundation::NSRange;
use objc2_metal::MTLBuffer;

/// An owned Metal buffer.
#[derive(Clone)]
pub struct Buffer {
    pub(super) inner: Retained<ProtocolObject<dyn MTLBuffer>>,
    storage: StorageMode,
}

impl Buffer {
    pub(super) const fn new(
        inner: Retained<ProtocolObject<dyn MTLBuffer>>,
        storage: StorageMode,
    ) -> Self {
        Self { inner, storage }
    }

    /// Returns the byte length of the buffer.
    #[must_use]
    pub fn length(&self) -> usize {
        self.inner.length()
    }

    /// Returns the allocation storage mode selected at creation time.
    #[must_use]
    pub const fn storage_mode(&self) -> StorageMode {
        self.storage
    }

    /// Copies bytes into CPU-visible storage after checking the complete range.
    pub fn write(&self, offset: usize, bytes: &[u8]) -> Result<(), Error> {
        if matches!(self.storage, StorageMode::Private | StorageMode::Memoryless) {
            return Err(Error::unsupported(
                "private and memoryless buffers are not CPU writable",
            ));
        }
        let end = offset
            .checked_add(bytes.len())
            .ok_or_else(|| Error::invalid_argument("buffer write range overflow"))?;
        if end > self.length() {
            return Err(Error::invalid_argument(
                "buffer write range is out of bounds",
            ));
        }
        if bytes.is_empty() {
            return Ok(());
        }
        let destination = self.inner.contents().as_ptr().cast::<u8>();
        // SAFETY: the storage mode is CPU-visible, the checked range is inside
        // the Metal allocation, and both pointers cover bytes.len() bytes.
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), destination.add(offset), bytes.len());
        }
        if self.storage == StorageMode::Managed {
            self.inner.didModifyRange(NSRange::new(offset, bytes.len()));
        }
        Ok(())
    }

    /// Copies shared-storage bytes into an owned vector after completion.
    pub fn read(&self, offset: usize, length: usize) -> Result<Vec<u8>, Error> {
        if self.storage != StorageMode::Shared {
            return Err(Error::unsupported(
                "CPU reads require shared storage after GPU completion",
            ));
        }
        let end = offset
            .checked_add(length)
            .ok_or_else(|| Error::invalid_argument("buffer read range overflow"))?;
        if end > self.length() {
            return Err(Error::invalid_argument(
                "buffer read range is out of bounds",
            ));
        }
        if length == 0 {
            return Ok(Vec::new());
        }
        let source = self.inner.contents().as_ptr().cast::<u8>();
        let mut bytes = vec![0_u8; length];
        // SAFETY: shared storage is CPU-visible, the checked range is inside
        // the Metal allocation, and the destination vector has `length` bytes.
        unsafe {
            std::ptr::copy_nonoverlapping(source.add(offset), bytes.as_mut_ptr(), length);
        }
        Ok(bytes)
    }
}
