//! Audited render and compute command encoder operations.

use crate::foundation::Error;
use crate::metal::{
    Buffer, CommandBuffer, ComputePipelineState, PrimitiveType, RenderPipelineState, Size, Viewport,
};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLCommandEncoder, MTLComputeCommandEncoder, MTLRenderCommandEncoder};
use std::marker::PhantomData;

/// A render encoder borrowing its command buffer until `end_encoding`.
pub struct RenderCommandEncoder<'a> {
    inner: Retained<ProtocolObject<dyn MTLRenderCommandEncoder>>,
    _command_buffer: PhantomData<&'a mut CommandBuffer>,
    ended: bool,
}

impl<'a> RenderCommandEncoder<'a> {
    pub(super) fn new(
        inner: Retained<ProtocolObject<dyn MTLRenderCommandEncoder>>,
        _command_buffer: &'a mut CommandBuffer,
    ) -> Self {
        Self {
            inner,
            _command_buffer: PhantomData,
            ended: false,
        }
    }

    /// Binds a render pipeline.
    pub fn set_pipeline(&self, pipeline: &RenderPipelineState) {
        self.inner.setRenderPipelineState(&pipeline.inner);
    }

    /// Binds a vertex buffer after checking the offset and binding index.
    pub fn set_vertex_buffer(
        &self,
        buffer: &Buffer,
        offset: usize,
        index: usize,
    ) -> Result<(), Error> {
        if index >= 31 || offset > buffer.length() {
            return Err(Error::invalid_argument(
                "vertex buffer binding or offset is out of bounds",
            ));
        }
        // SAFETY: the buffer is retained by the caller for the duration of the
        // encoder, and the checked offset is within its allocation.
        unsafe {
            self.inner
                .setVertexBuffer_offset_atIndex(Some(&buffer.inner), offset, index);
        }
        Ok(())
    }

    /// Sets the viewport after rejecting non-finite or negative dimensions.
    pub fn set_viewport(&self, viewport: Viewport) -> Result<(), Error> {
        if ![
            viewport.origin_x,
            viewport.origin_y,
            viewport.width,
            viewport.height,
            viewport.z_near,
            viewport.z_far,
        ]
        .into_iter()
        .all(f64::is_finite)
            || viewport.width < 0.0
            || viewport.height < 0.0
        {
            return Err(Error::invalid_argument("viewport contains invalid values"));
        }
        self.inner.setViewport(viewport.into());
        Ok(())
    }

    /// Encodes a non-indexed primitive draw.
    pub fn draw_primitives(
        &self,
        primitive: PrimitiveType,
        vertex_start: usize,
        vertex_count: usize,
    ) -> Result<(), Error> {
        if vertex_count == 0 {
            return Err(Error::invalid_argument("vertex count must be non-zero"));
        }
        // SAFETY: the public method only passes non-zero usize counts and the
        // Metal encoder performs the remaining pipeline/resource validation.
        unsafe {
            self.inner.drawPrimitives_vertexStart_vertexCount(
                primitive.as_objc(),
                vertex_start,
                vertex_count,
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

impl Drop for RenderCommandEncoder<'_> {
    fn drop(&mut self) {
        if !self.ended {
            self.inner.endEncoding();
        }
    }
}

/// A compute encoder borrowing its command buffer until `end_encoding`.
pub struct ComputeCommandEncoder<'a> {
    inner: Retained<ProtocolObject<dyn MTLComputeCommandEncoder>>,
    _command_buffer: PhantomData<&'a mut CommandBuffer>,
    ended: bool,
}

impl<'a> ComputeCommandEncoder<'a> {
    pub(super) fn new(
        inner: Retained<ProtocolObject<dyn MTLComputeCommandEncoder>>,
        _command_buffer: &'a mut CommandBuffer,
    ) -> Self {
        Self {
            inner,
            _command_buffer: PhantomData,
            ended: false,
        }
    }

    /// Binds a compute pipeline.
    pub fn set_pipeline(&self, pipeline: &ComputePipelineState) {
        self.inner.setComputePipelineState(&pipeline.inner);
    }

    /// Binds a compute buffer after checking the offset and binding index.
    pub fn set_buffer(&self, buffer: &Buffer, offset: usize, index: usize) -> Result<(), Error> {
        if index >= 31 || offset > buffer.length() {
            return Err(Error::invalid_argument(
                "compute buffer binding or offset is out of bounds",
            ));
        }
        // SAFETY: the buffer is retained by the caller for the duration of the
        // encoder, and the checked offset is within its allocation.
        unsafe {
            self.inner
                .setBuffer_offset_atIndex(Some(&buffer.inner), offset, index);
        }
        Ok(())
    }

    /// Dispatches a one-dimensional grid.
    pub fn dispatch_1d(&self, threads: usize, threads_per_group: usize) -> Result<(), Error> {
        if threads == 0 || threads_per_group == 0 {
            return Err(Error::invalid_argument(
                "dispatch dimensions must be non-zero",
            ));
        }
        self.inner.dispatchThreads_threadsPerThreadgroup(
            Size::new(threads, 1, 1).into(),
            Size::new(threads_per_group, 1, 1).into(),
        );
        Ok(())
    }

    /// Ends encoding and releases the borrow of the command buffer.
    pub fn end_encoding(mut self) {
        self.inner.endEncoding();
        self.ended = true;
    }
}

impl Drop for ComputeCommandEncoder<'_> {
    fn drop(&mut self) {
        if !self.ended {
            self.inner.endEncoding();
        }
    }
}
