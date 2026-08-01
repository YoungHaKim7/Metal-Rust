//! Safe Metal shader library and pipeline objects.

use super::PixelFormat;
use crate::Error;

/// An owned Metal shader library.
#[derive(Clone)]
pub struct Library {
    pub(crate) inner: metal_rust_ffi::Library,
}

impl Library {
    pub(crate) const fn from_ffi(inner: metal_rust_ffi::Library) -> Self {
        Self { inner }
    }

    /// Looks up a named shader function.
    pub fn function(&self, name: &str) -> Result<Function, Error> {
        self.inner
            .function(name)
            .map(|inner| Function { inner })
            .map_err(Error::from_ffi)
    }
}

/// A named shader function.
#[derive(Clone)]
pub struct Function {
    pub(crate) inner: metal_rust_ffi::Function,
}

impl Function {
    /// Returns the function's Metal name.
    #[must_use]
    pub fn name(&self) -> String {
        self.inner.name()
    }
}

/// Options for compiling Metal Shading Language.
pub struct CompileOptions {
    pub(crate) inner: metal_rust_ffi::CompileOptions,
}

impl CompileOptions {
    /// Creates strict, non-fast-math compiler options.
    #[must_use]
    pub fn strict() -> Self {
        Self {
            inner: metal_rust_ffi::CompileOptions::strict(),
        }
    }
}

/// A compiled render pipeline state.
#[derive(Clone)]
pub struct RenderPipelineState {
    pub(crate) inner: metal_rust_ffi::RenderPipelineState,
}

impl RenderPipelineState {
    /// Returns the maximum threadgroup size reported by Metal.
    #[must_use]
    pub fn max_total_threads_per_threadgroup(&self) -> usize {
        self.inner.max_total_threads_per_threadgroup()
    }
}

/// A compiled compute pipeline state.
#[derive(Clone)]
pub struct ComputePipelineState {
    pub(crate) inner: metal_rust_ffi::ComputePipelineState,
}

impl ComputePipelineState {
    /// Returns the execution width reported by Metal.
    #[must_use]
    pub fn execution_width(&self) -> usize {
        self.inner.execution_width()
    }

    /// Returns the maximum threadgroup size reported by Metal.
    #[must_use]
    pub fn max_total_threads_per_threadgroup(&self) -> usize {
        self.inner.max_total_threads_per_threadgroup()
    }
}

/// A render pipeline descriptor.
pub struct RenderPipelineDescriptor {
    pub(crate) inner: metal_rust_ffi::RenderPipelineDescriptor,
}

impl RenderPipelineDescriptor {
    /// Creates a descriptor from vertex/fragment functions and a color format.
    #[must_use]
    pub fn new(vertex: &Function, fragment: Option<&Function>, color_format: PixelFormat) -> Self {
        Self {
            inner: metal_rust_ffi::RenderPipelineDescriptor::new(
                &vertex.inner,
                fragment.map(|value| &value.inner),
                color_format,
            ),
        }
    }
}
