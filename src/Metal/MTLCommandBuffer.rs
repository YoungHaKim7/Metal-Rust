//! Safe Metal command buffer lifecycle.

use super::{
    BlitCommandEncoder, CommandBufferStatus, ComputeCommandEncoder, ComputePassDescriptor,
    RenderCommandEncoder, RenderPassDescriptor,
};
use crate::Error;
use std::marker::PhantomData;

/// A proof that a command buffer has completed synchronously.
pub struct CompletionToken<'a> {
    pub(crate) _command_buffer: PhantomData<&'a CommandBuffer>,
}

/// An owned Metal command buffer.
pub struct CommandBuffer {
    pub(crate) inner: metal_rust_ffi::CommandBuffer,
}

impl CommandBuffer {
    pub(crate) const fn from_ffi(inner: metal_rust_ffi::CommandBuffer) -> Self {
        Self { inner }
    }

    /// Begins a render encoder. The exclusive borrow prevents command
    /// submission while the encoder is active.
    pub fn render_encoder<'a>(
        &'a mut self,
        descriptor: &RenderPassDescriptor,
    ) -> Result<RenderCommandEncoder<'a>, Error> {
        self.inner
            .render_encoder(&descriptor.inner)
            .map(|inner| RenderCommandEncoder { inner })
            .map_err(Error::from_ffi)
    }

    /// Begins a compute encoder. The exclusive borrow prevents command
    /// submission while the encoder is active.
    pub fn compute_encoder<'a>(
        &'a mut self,
        descriptor: &ComputePassDescriptor,
    ) -> Result<ComputeCommandEncoder<'a>, Error> {
        self.inner
            .compute_encoder(&descriptor.inner)
            .map(|inner| ComputeCommandEncoder { inner })
            .map_err(Error::from_ffi)
    }

    /// Begins a blit encoder. The exclusive borrow prevents concurrent
    /// command-buffer mutation while the encoder is active.
    pub fn blit_encoder<'a>(&'a mut self) -> Result<BlitCommandEncoder<'a>, Error> {
        self.inner
            .blit_encoder()
            .map(|inner| BlitCommandEncoder { inner })
            .map_err(Error::from_ffi)
    }

    /// Commits the command buffer for execution.
    pub fn commit(&self) {
        self.inner.commit();
    }

    /// Waits synchronously until the command buffer completes and returns a
    /// token required for GPU-written resource readback.
    pub fn wait_until_completed(&self) -> CompletionToken<'_> {
        self.inner.wait_until_completed();
        CompletionToken {
            _command_buffer: PhantomData,
        }
    }

    /// Returns the current command buffer state.
    #[must_use]
    pub fn status(&self) -> CommandBufferStatus {
        self.inner.status()
    }

    /// Returns the native execution error, if the buffer failed.
    #[must_use]
    pub fn error(&self) -> Option<Error> {
        self.inner.error().map(Error::from_ffi)
    }
}
