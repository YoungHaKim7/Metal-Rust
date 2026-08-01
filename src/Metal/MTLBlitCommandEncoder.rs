//! Safe buffer-copy command encoder.

use super::Buffer;
use crate::Error;

/// A blit encoder that borrows its command buffer until encoding ends.
pub struct BlitCommandEncoder<'a> {
    pub(crate) inner: metal_rust_ffi::BlitCommandEncoder<'a>,
}

impl BlitCommandEncoder<'_> {
    /// Copies a checked byte range between two buffers.
    pub fn copy_buffer(
        &self,
        source: &Buffer,
        source_offset: usize,
        destination: &Buffer,
        destination_offset: usize,
        size: usize,
    ) -> Result<(), Error> {
        self.inner
            .copy_buffer(
                &source.inner,
                source_offset,
                &destination.inner,
                destination_offset,
                size,
            )
            .map_err(Error::from_ffi)
    }

    /// Ends encoding explicitly. Dropping the encoder also ends encoding.
    pub fn end_encoding(self) {
        self.inner.end_encoding();
    }
}
