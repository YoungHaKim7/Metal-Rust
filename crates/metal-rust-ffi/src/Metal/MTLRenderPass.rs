//! Audited render-pass descriptor construction.

use crate::foundation::Error;
use crate::metal::{ClearColor, Texture};
use objc2::rc::Retained;
use objc2_metal::{MTLClearColor, MTLLoadAction, MTLRenderPassDescriptor, MTLStoreAction};

/// A render-pass descriptor whose attachment indices are checked before FFI.
pub struct RenderPassDescriptor {
    pub(super) inner: Retained<MTLRenderPassDescriptor>,
}

impl RenderPassDescriptor {
    /// Creates an empty render-pass descriptor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: MTLRenderPassDescriptor::renderPassDescriptor(),
        }
    }

    /// Sets a color attachment's texture and clear color.
    pub fn set_color_attachment(
        &self,
        index: usize,
        texture: &Texture,
        clear_color: ClearColor,
    ) -> Result<(), Error> {
        if index >= 8 {
            return Err(Error::invalid_argument(
                "Metal supports at most eight color attachments",
            ));
        }
        let attachments = self.inner.colorAttachments();
        // SAFETY: Metal's color attachment array has eight entries and index is
        // checked against that fixed API limit.
        let attachment = unsafe { attachments.objectAtIndexedSubscript(index) };
        attachment.setTexture(Some(&texture.inner));
        attachment.setLoadAction(MTLLoadAction::Clear);
        attachment.setStoreAction(MTLStoreAction::Store);
        attachment.setClearColor(MTLClearColor {
            red: clear_color.red,
            green: clear_color.green,
            blue: clear_color.blue,
            alpha: clear_color.alpha,
        });
        Ok(())
    }
}

impl Default for RenderPassDescriptor {
    fn default() -> Self {
        Self::new()
    }
}

/// A compute-pass descriptor.
pub struct ComputePassDescriptor {
    pub(super) inner: Retained<objc2_metal::MTLComputePassDescriptor>,
}

impl ComputePassDescriptor {
    /// Creates a default compute-pass descriptor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: objc2_metal::MTLComputePassDescriptor::computePassDescriptor(),
        }
    }
}

impl Default for ComputePassDescriptor {
    fn default() -> Self {
        Self::new()
    }
}
