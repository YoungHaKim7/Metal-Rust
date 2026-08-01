//! Audited shader library and pipeline creation boundary.

use crate::foundation::Error;
use crate::metal::PixelFormat;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_foundation::NSString;
use objc2_metal::{
    MTLCompileOptions, MTLComputePipelineState, MTLFunction, MTLLibrary,
    MTLRenderPipelineColorAttachmentDescriptorArray, MTLRenderPipelineDescriptor,
    MTLRenderPipelineState,
};

/// An owned Metal shader library.
#[derive(Clone)]
pub struct Library {
    pub(super) inner: Retained<ProtocolObject<dyn MTLLibrary>>,
}

impl Library {
    pub(super) const fn new(inner: Retained<ProtocolObject<dyn MTLLibrary>>) -> Self {
        Self { inner }
    }

    /// Looks up a named shader function.
    pub fn function(&self, name: &str) -> Result<Function, Error> {
        if name.is_empty() || name.as_bytes().contains(&0) {
            return Err(Error::invalid_argument("shader function name is invalid"));
        }
        let name = NSString::from_str(name);
        self.inner
            .newFunctionWithName(&name)
            .map(Function::new)
            .ok_or_else(|| Error::unsupported("the shader function was not found"))
    }
}

/// A named function inside a Metal library.
#[derive(Clone)]
pub struct Function {
    pub(super) inner: Retained<ProtocolObject<dyn MTLFunction>>,
}

impl Function {
    pub(super) const fn new(inner: Retained<ProtocolObject<dyn MTLFunction>>) -> Self {
        Self { inner }
    }

    /// Returns the function's Metal name.
    #[must_use]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }
}

/// A compiled render pipeline state.
#[derive(Clone)]
pub struct RenderPipelineState {
    pub(super) inner: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
}

impl RenderPipelineState {
    pub(super) const fn new(inner: Retained<ProtocolObject<dyn MTLRenderPipelineState>>) -> Self {
        Self { inner }
    }

    /// Returns the maximum number of threads per threadgroup.
    #[must_use]
    pub fn max_total_threads_per_threadgroup(&self) -> usize {
        self.inner.maxTotalThreadsPerThreadgroup()
    }
}

/// A compiled compute pipeline state.
#[derive(Clone)]
pub struct ComputePipelineState {
    pub(super) inner: Retained<ProtocolObject<dyn MTLComputePipelineState>>,
}

impl ComputePipelineState {
    pub(super) const fn new(inner: Retained<ProtocolObject<dyn MTLComputePipelineState>>) -> Self {
        Self { inner }
    }

    /// Returns the execution width of the compute pipeline.
    #[must_use]
    pub fn execution_width(&self) -> usize {
        self.inner.threadExecutionWidth()
    }

    /// Returns the maximum number of threads per threadgroup.
    #[must_use]
    pub fn max_total_threads_per_threadgroup(&self) -> usize {
        self.inner.maxTotalThreadsPerThreadgroup()
    }
}

/// A validated render-pipeline descriptor.
pub struct RenderPipelineDescriptor {
    pub(super) inner: Retained<MTLRenderPipelineDescriptor>,
}

impl RenderPipelineDescriptor {
    /// Creates a descriptor from a vertex and fragment function.
    #[must_use]
    pub fn new(vertex: &Function, fragment: Option<&Function>, color_format: PixelFormat) -> Self {
        let inner = MTLRenderPipelineDescriptor::new();
        inner.setVertexFunction(Some(&vertex.inner));
        inner.setFragmentFunction(fragment.map(|function| &*function.inner));
        inner.setRasterSampleCount(1);
        let attachments: Retained<MTLRenderPipelineColorAttachmentDescriptorArray> =
            inner.colorAttachments();
        // SAFETY: the Metal render pipeline color attachment array has eight
        // entries; index zero is always valid.
        let attachment = unsafe { attachments.objectAtIndexedSubscript(0) };
        attachment.setPixelFormat(color_format.as_objc());
        Self { inner }
    }
}

/// A validated compile-options object.
pub struct CompileOptions {
    pub(super) inner: Retained<MTLCompileOptions>,
}

impl CompileOptions {
    /// Creates options with fast-math explicitly disabled.
    #[must_use]
    pub fn strict() -> Self {
        let inner = MTLCompileOptions::new();
        inner.setMathMode(objc2_metal::MTLMathMode::Safe);
        Self { inner }
    }
}
