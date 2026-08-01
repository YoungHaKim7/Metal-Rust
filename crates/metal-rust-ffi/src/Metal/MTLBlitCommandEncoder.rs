//! Audited Metal blit encoder operations.

use crate::foundation::Error;
use crate::metal::{Buffer, CommandBuffer};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLBlitCommandEncoder, MTLCommandEncoder};
use std::marker::PhantomData;

/// A blit encoder borrowing its command buffer until `end_encoding`.
pub struct BlitCommandEncoder<'a> {
    inner: Retained<ProtocolObject<dyn MTLBlitCommandEncoder>>,
    _command_buffer: PhantomData<&'a mut CommandBuffer>,
    ended: bool,
}

impl<'a> BlitCommandEncoder<'a> {
    pub(super) fn new(
        inner: Retained<ProtocolObject<dyn MTLBlitCommandEncoder>>,
        _command_buffer: &'a mut CommandBuffer,
    ) -> Self {
        Self {
            inner,
            _command_buffer: PhantomData,
            ended: false,
        }
    }

    /// Copies a checked byte range between two buffers.
    pub fn copy_buffer(
        &self,
        source: &Buffer,
        source_offset: usize,
        destination: &Buffer,
        destination_offset: usize,
        size: usize,
    ) -> Result<(), Error> {
        let source_end = source_offset
            .checked_add(size)
            .ok_or_else(|| Error::invalid_argument("source copy range overflow"))?;
        let destination_end = destination_offset
            .checked_add(size)
            .ok_or_else(|| Error::invalid_argument("destination copy range overflow"))?;
        if source_end > source.length() || destination_end > destination.length() {
            return Err(Error::invalid_argument(
                "buffer copy range is out of bounds",
            ));
        }
        if size == 0 {
            return Ok(());
        }
        // SAFETY: both buffers are retained for the encoder lifetime, and the
        // checked offsets and size are inside their Metal allocations.
        unsafe {
            self.inner
                .copyFromBuffer_sourceOffset_toBuffer_destinationOffset_size(
                    &source.inner,
                    source_offset,
                    &destination.inner,
                    destination_offset,
                    size,
                );
        }
        Ok(())
    }

    /// Ends encoding and releases the borrow of the command buffer.
    pub fn end_encoding(mut self) {
        self.inner.endEncoding();
        self.ended = true;
    }
}

impl Drop for BlitCommandEncoder<'_> {
    fn drop(&mut self) {
        if !self.ended {
            self.inner.endEncoding();
        }
    }
}
