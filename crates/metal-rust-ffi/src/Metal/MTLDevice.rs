use crate::foundation::Error;
use crate::metal::{
    Buffer, CommandQueue, CompileOptions, ComputePipelineState, Library, MAX_TIMESTAMP_COUNTERS,
    RenderPipelineDescriptor, RenderPipelineState, ResourceOptions, Texture, TextureDescriptor,
    TimestampCounterHeap,
};
use objc2::rc::Retained;
use objc2::runtime::{NSObjectProtocol, ProtocolObject};
use objc2::sel;
use objc2_foundation::NSString;
use objc2_metal::{
    MTL4CounterHeapDescriptor, MTL4CounterHeapType, MTLCreateSystemDefaultDevice, MTLDevice,
};

#[derive(Clone)]
/// An owned Metal device protocol object.
pub struct Device {
    pub(crate) inner: Retained<ProtocolObject<dyn MTLDevice>>,
}

impl Device {
    pub(crate) const fn from_inner(inner: Retained<ProtocolObject<dyn MTLDevice>>) -> Self {
        Self { inner }
    }

    #[must_use]
    /// Returns the system default device, if one exists.
    pub fn system_default() -> Option<Self> {
        MTLCreateSystemDefaultDevice().map(|inner| Self { inner })
    }

    #[must_use]
    /// Returns the device name as an owned Rust string.
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    /// Creates a command queue, optionally with a checked upper bound.
    pub fn new_command_queue(
        &self,
        max_command_buffers: Option<usize>,
    ) -> Result<CommandQueue, Error> {
        let inner = match max_command_buffers {
            None => self.inner.newCommandQueue(),
            Some(count) if count > 0 => self.inner.newCommandQueueWithMaxCommandBufferCount(count),
            Some(_) => {
                return Err(Error::invalid_argument(
                    "command buffer count must be non-zero",
                ));
            }
        };
        inner
            .map(CommandQueue::new)
            .ok_or_else(|| Error::unsupported("Metal could not create a command queue"))
    }

    /// Allocates a buffer with the selected storage mode.
    pub fn new_buffer(&self, length: usize, options: ResourceOptions) -> Result<Buffer, Error> {
        if length == 0 {
            return Err(Error::invalid_argument("buffer length must be non-zero"));
        }
        self.inner
            .newBufferWithLength_options(length, options.as_objc())
            .map(|inner| Buffer::new(inner, options.storage_mode()))
            .ok_or_else(|| Error::unsupported("Metal could not allocate the buffer"))
    }

    /// Allocates a texture from a validated descriptor.
    pub fn new_texture(&self, descriptor: &TextureDescriptor) -> Result<Texture, Error> {
        self.inner
            .newTextureWithDescriptor(&descriptor.inner)
            .map(Texture::new)
            .ok_or_else(|| Error::unsupported("Metal could not allocate the texture"))
    }

    /// Compiles Metal Shading Language source into an owned library.
    pub fn new_library_from_source(
        &self,
        source: &str,
        options: Option<&CompileOptions>,
    ) -> Result<Library, Error> {
        if source.is_empty() || source.as_bytes().contains(&0) {
            return Err(Error::invalid_argument("Metal source is invalid"));
        }
        let source = NSString::from_str(source);
        self.inner
            .newLibraryWithSource_options_error(&source, options.map(|value| &*value.inner))
            .map(Library::new)
            .map_err(|error| crate::foundation::metal_error(&error))
    }

    /// Builds a render pipeline from a validated descriptor.
    pub fn new_render_pipeline(
        &self,
        descriptor: &RenderPipelineDescriptor,
    ) -> Result<RenderPipelineState, Error> {
        self.inner
            .newRenderPipelineStateWithDescriptor_error(&descriptor.inner)
            .map(RenderPipelineState::new)
            .map_err(|error| crate::foundation::metal_error(&error))
    }

    /// Builds a compute pipeline from a named library function.
    pub fn new_compute_pipeline(
        &self,
        function: &crate::metal::Function,
    ) -> Result<ComputePipelineState, Error> {
        self.inner
            .newComputePipelineStateWithFunction_error(&function.inner)
            .map(ComputePipelineState::new)
            .map_err(|error| crate::foundation::metal_error(&error))
    }

    /// Returns whether a texture sample count is supported by the device.
    #[must_use]
    pub fn supports_texture_sample_count(&self, sample_count: usize) -> bool {
        sample_count > 0 && self.inner.supportsTextureSampleCount(sample_count)
    }

    /// Creates a Metal 4 timestamp counter heap after capability checks.
    pub fn new_timestamp_counter_heap(&self, count: usize) -> Result<TimestampCounterHeap, Error> {
        if !(1..=MAX_TIMESTAMP_COUNTERS).contains(&count) {
            return Err(Error::unsupported(format!(
                "timestamp counter count must be between 1 and {MAX_TIMESTAMP_COUNTERS}"
            )));
        }
        if !self
            .inner
            .respondsToSelector(sel!(newCounterHeapWithDescriptor:error:))
        {
            return Err(Error::unsupported(
                "MTL4CounterHeap requires macOS 26 or newer",
            ));
        }

        let descriptor = MTL4CounterHeapDescriptor::new();
        descriptor.setType(MTL4CounterHeapType::Timestamp);
        // SAFETY: The public facade restricts count to the range validated by
        // Metal's current timestamp CounterHeap implementation.
        unsafe { descriptor.setCount(count) };
        self.inner
            .newCounterHeapWithDescriptor_error(&descriptor)
            .map(|inner| TimestampCounterHeap::new(inner, count))
            .map_err(|error| crate::foundation::metal_error(&error))
    }
}
