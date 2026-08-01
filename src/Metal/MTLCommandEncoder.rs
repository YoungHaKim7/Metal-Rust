//! Safe render and compute command encoders.

use super::{Buffer, ComputePipelineState, PrimitiveType, RenderPipelineState, Viewport};
use crate::Error;

/// A render encoder that borrows its command buffer until encoding ends.
pub struct RenderCommandEncoder<'a> {
    pub(crate) inner: metal_rust_ffi::RenderCommandEncoder<'a>,
}

impl RenderCommandEncoder<'_> {
    /// Binds a render pipeline.
    pub fn set_pipeline(&self, pipeline: &RenderPipelineState) {
        self.inner.set_pipeline(&pipeline.inner);
    }

    /// Binds a vertex buffer with checked offset and binding index.
    pub fn set_vertex_buffer(
        &self,
        buffer: &Buffer,
        offset: usize,
        index: usize,
    ) -> Result<(), Error> {
        self.inner
            .set_vertex_buffer(&buffer.inner, offset, index)
            .map_err(Error::from_ffi)
    }

    /// Sets a validated viewport.
    pub fn set_viewport(&self, viewport: Viewport) -> Result<(), Error> {
        self.inner.set_viewport(viewport).map_err(Error::from_ffi)
    }

    /// Encodes a non-indexed primitive draw.
    pub fn draw_primitives(
        &self,
        primitive: PrimitiveType,
        vertex_start: usize,
        vertex_count: usize,
    ) -> Result<(), Error> {
        self.inner
            .draw_primitives(primitive, vertex_start, vertex_count)
            .map_err(Error::from_ffi)
    }

    /// Ends encoding explicitly. Dropping the encoder also ends encoding.
    pub fn end_encoding(self) {
        self.inner.end_encoding();
    }
}

/// A compute encoder that borrows its command buffer until encoding ends.
pub struct ComputeCommandEncoder<'a> {
    pub(crate) inner: metal_rust_ffi::ComputeCommandEncoder<'a>,
}

impl ComputeCommandEncoder<'_> {
    /// Binds a compute pipeline.
    pub fn set_pipeline(&self, pipeline: &ComputePipelineState) {
        self.inner.set_pipeline(&pipeline.inner);
    }

    /// Binds a compute buffer with checked offset and binding index.
    pub fn set_buffer(&self, buffer: &Buffer, offset: usize, index: usize) -> Result<(), Error> {
        self.inner
            .set_buffer(&buffer.inner, offset, index)
            .map_err(Error::from_ffi)
    }

    /// Dispatches a one-dimensional grid.
    pub fn dispatch_1d(&self, threads: usize, threads_per_group: usize) -> Result<(), Error> {
        self.inner
            .dispatch_1d(threads, threads_per_group)
            .map_err(Error::from_ffi)
    }

    /// Ends encoding explicitly. Dropping the encoder also ends encoding.
    pub fn end_encoding(self) {
        self.inner.end_encoding();
    }
}
