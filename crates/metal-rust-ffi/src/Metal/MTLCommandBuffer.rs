//! Audited command buffer lifecycle and encoder borrowing boundary.

use crate::foundation::{Error, metal_error};
use crate::metal::{
    BlitCommandEncoder, ComputeCommandEncoder, ComputePassDescriptor, RenderCommandEncoder,
    RenderPassDescriptor,
};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::MTLCommandBuffer;

/// An owned Metal command buffer.
pub struct CommandBuffer {
    pub(crate) inner: Retained<ProtocolObject<dyn MTLCommandBuffer>>,
}

impl CommandBuffer {
    pub(super) const fn new(inner: Retained<ProtocolObject<dyn MTLCommandBuffer>>) -> Self {
        Self { inner }
    }

    /// Begins a render encoder while borrowing the command buffer exclusively.
    pub fn render_encoder<'a>(
        &'a mut self,
        descriptor: &RenderPassDescriptor,
    ) -> Result<RenderCommandEncoder<'a>, Error> {
        self.inner
            .renderCommandEncoderWithDescriptor(&descriptor.inner)
            .map(|inner| RenderCommandEncoder::new(inner, self))
            .ok_or_else(|| Error::unsupported("Metal could not create a render encoder"))
    }

    /// Begins a compute encoder while borrowing the command buffer exclusively.
    pub fn compute_encoder<'a>(
        &'a mut self,
        descriptor: &ComputePassDescriptor,
    ) -> Result<ComputeCommandEncoder<'a>, Error> {
        self.inner
            .computeCommandEncoderWithDescriptor(&descriptor.inner)
            .map(|inner| ComputeCommandEncoder::new(inner, self))
            .ok_or_else(|| Error::unsupported("Metal could not create a compute encoder"))
    }

    /// Begins a blit encoder while borrowing the command buffer exclusively.
    pub fn blit_encoder<'a>(&'a mut self) -> Result<BlitCommandEncoder<'a>, Error> {
        self.inner
            .blitCommandEncoder()
            .map(|inner| BlitCommandEncoder::new(inner, self))
            .ok_or_else(|| Error::unsupported("Metal could not create a blit encoder"))
    }

    /// Commits the command buffer for execution.
    pub fn commit(&self) {
        self.inner.commit();
    }

    /// Waits synchronously until the command buffer completes.
    pub fn wait_until_completed(&self) {
        self.inner.waitUntilCompleted();
    }

    /// Returns the current command buffer state.
    #[must_use]
    pub fn status(&self) -> crate::metal::CommandBufferStatus {
        self.inner.status().into()
    }
    /// Returns the native execution error, if the buffer failed.
    #[must_use]
    pub fn error(&self) -> Option<Error> {
        self.inner.error().map(|error| metal_error(&error))
    }
}
