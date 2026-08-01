use super::{
    Buffer, CommandQueue, CompileOptions, ComputePipelineState, Function, Library,
    RenderPipelineDescriptor, RenderPipelineState, ResourceOptions, Texture, TextureDescriptor,
    TimestampCounterHeap,
};
use crate::Error;

/// An owned reference to an Apple Metal device.
#[derive(Clone)]
pub struct Device {
    pub(crate) inner: metal_rust_ffi::Device,
}

impl Device {
    pub(crate) const fn from_ffi(inner: metal_rust_ffi::Device) -> Self {
        Self { inner }
    }

    /// Returns the preferred system Metal device, if one is available.
    #[must_use]
    pub fn system_default() -> Option<Self> {
        metal_rust_ffi::Device::system_default().map(|inner| Self { inner })
    }

    /// Returns the device's user-visible name as an owned Rust string.
    #[must_use]
    pub fn name(&self) -> String {
        self.inner.name()
    }

    /// Creates a command queue, optionally with a checked upper bound.
    pub fn new_command_queue(
        &self,
        max_command_buffers: Option<usize>,
    ) -> Result<CommandQueue, Error> {
        self.inner
            .new_command_queue(max_command_buffers)
            .map(CommandQueue::from_ffi)
            .map_err(Error::from_ffi)
    }

    /// Allocates a buffer with the selected resource options.
    pub fn new_buffer(&self, length: usize, options: ResourceOptions) -> Result<Buffer, Error> {
        self.inner
            .new_buffer(length, options)
            .map(Buffer::from_ffi)
            .map_err(Error::from_ffi)
    }

    /// Allocates a texture from a validated descriptor.
    pub fn new_texture(&self, descriptor: &TextureDescriptor) -> Result<Texture, Error> {
        self.inner
            .new_texture(&descriptor.inner)
            .map(Texture::from_ffi)
            .map_err(Error::from_ffi)
    }

    /// Compiles Metal Shading Language source.
    pub fn new_library_from_source(
        &self,
        source: &str,
        options: Option<&CompileOptions>,
    ) -> Result<Library, Error> {
        self.inner
            .new_library_from_source(source, options.map(|value| &value.inner))
            .map(Library::from_ffi)
            .map_err(Error::from_ffi)
    }

    /// Builds a render pipeline.
    pub fn new_render_pipeline(
        &self,
        descriptor: &RenderPipelineDescriptor,
    ) -> Result<RenderPipelineState, Error> {
        self.inner
            .new_render_pipeline(&descriptor.inner)
            .map(|inner| RenderPipelineState { inner })
            .map_err(Error::from_ffi)
    }

    /// Builds a compute pipeline from a library function.
    pub fn new_compute_pipeline(&self, function: &Function) -> Result<ComputePipelineState, Error> {
        self.inner
            .new_compute_pipeline(&function.inner)
            .map(|inner| ComputePipelineState { inner })
            .map_err(Error::from_ffi)
    }

    /// Returns whether a texture sample count is supported.
    #[must_use]
    pub fn supports_texture_sample_count(&self, sample_count: usize) -> bool {
        self.inner.supports_texture_sample_count(sample_count)
    }

    /// Creates a Metal 4 timestamp counter heap.
    ///
    /// The count is validated before the Objective-C boundary is entered.
    pub fn new_timestamp_counter_heap(&self, count: usize) -> Result<TimestampCounterHeap, Error> {
        if count == 0 || count > crate::MAX_TIMESTAMP_COUNTERS {
            return Err(Error::invalid_argument(format!(
                "timestamp counter count must be between 1 and {}",
                crate::MAX_TIMESTAMP_COUNTERS
            )));
        }
        self.inner
            .new_timestamp_counter_heap(count)
            .map(TimestampCounterHeap::from_ffi)
            .map_err(Error::from_ffi)
    }
}
