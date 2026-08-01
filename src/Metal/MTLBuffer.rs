//! Safe Metal buffer resources.

use super::{CompletionToken, StorageMode};
use crate::Error;

/// An owned Metal buffer.
#[derive(Clone)]
pub struct Buffer {
    pub(crate) inner: metal_rust_ffi::Buffer,
}

impl Buffer {
    pub(crate) const fn from_ffi(inner: metal_rust_ffi::Buffer) -> Self {
        Self { inner }
    }

    /// Returns the byte length of the buffer.
    #[must_use]
    pub fn length(&self) -> usize {
        self.inner.length()
    }

    /// Returns the storage mode selected when the buffer was allocated.
    #[must_use]
    pub const fn storage_mode(&self) -> StorageMode {
        self.inner.storage_mode()
    }

    /// Copies bytes into CPU-visible storage after checking the complete range.
    pub fn write(&self, offset: usize, bytes: &[u8]) -> Result<(), Error> {
        self.inner.write(offset, bytes).map_err(Error::from_ffi)
    }

    /// Copies shared-storage bytes into an owned vector after a completion
    /// token. Managed and private storage require an explicit sync model.
    pub fn read(
        &self,
        offset: usize,
        length: usize,
        _completion: &CompletionToken<'_>,
    ) -> Result<Vec<u8>, Error> {
        self.inner.read(offset, length).map_err(Error::from_ffi)
    }
}
