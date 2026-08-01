//! Audited command queue ownership and submission boundary.

use crate::foundation::Error;
use crate::metal::{CommandBuffer, Device};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::MTLCommandQueue;

/// An owned Metal command queue.
pub struct CommandQueue {
    pub(super) inner: Retained<ProtocolObject<dyn MTLCommandQueue>>,
}

impl CommandQueue {
    pub(super) const fn new(inner: Retained<ProtocolObject<dyn MTLCommandQueue>>) -> Self {
        Self { inner }
    }

    /// Returns the device that created this queue.
    #[must_use]
    pub fn device(&self) -> Device {
        Device::from_inner(self.inner.device())
    }

    /// Returns the optional debug label.
    #[must_use]
    pub fn label(&self) -> Option<String> {
        self.inner.label().map(|label| label.to_string())
    }

    /// Sets a debug label.
    pub fn set_label(&self, label: Option<&str>) {
        let label = label.map(objc2_foundation::NSString::from_str);
        self.inner.setLabel(label.as_deref());
    }

    /// Creates a retained command buffer with normal resource retention.
    pub fn command_buffer(&self) -> Result<CommandBuffer, Error> {
        self.inner
            .commandBuffer()
            .map(CommandBuffer::new)
            .ok_or_else(|| Error::unsupported("Metal could not create a command buffer"))
    }
}
