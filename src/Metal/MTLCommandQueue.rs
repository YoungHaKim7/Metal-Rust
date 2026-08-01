//! Safe Metal command queues.

use super::{CommandBuffer, Device};
use crate::Error;

/// An owned Metal command queue.
pub struct CommandQueue {
    pub(crate) inner: metal_rust_ffi::CommandQueue,
}

impl CommandQueue {
    pub(crate) const fn from_ffi(inner: metal_rust_ffi::CommandQueue) -> Self {
        Self { inner }
    }

    /// Returns the device that created this queue.
    #[must_use]
    pub fn device(&self) -> Device {
        Device::from_ffi(self.inner.device())
    }

    /// Returns the optional debug label.
    #[must_use]
    pub fn label(&self) -> Option<String> {
        self.inner.label()
    }

    /// Sets a debug label.
    pub fn set_label(&self, label: Option<&str>) {
        self.inner.set_label(label);
    }

    /// Creates a command buffer with normal resource retention.
    pub fn command_buffer(&self) -> Result<CommandBuffer, Error> {
        self.inner
            .command_buffer()
            .map(CommandBuffer::from_ffi)
            .map_err(Error::from_ffi)
    }
}
