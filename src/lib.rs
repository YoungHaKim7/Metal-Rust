#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Safe Rust interfaces for Apple Metal.
//!
//! Raw Objective-C objects and unsafe operations are confined to the audited
//! `metal-rust-ffi` implementation crate. This public crate never exposes
//! object pointers or requires callers to uphold Objective-C runtime
//! invariants.
//!
//! The following visibility fixture must fail to compile: an application can
//! own a device, but cannot extract its implementation object or bypass the
//! safe boundary.
//!
//! ```compile_fail
//! let device = metal_rust::Device::system_default().unwrap();
//! let _implementation_object = device.inner;
//! ```

#[path = "Foundation/Foundation.rs"]
pub mod foundation;
#[path = "Metal/Metal.rs"]
pub mod metal;
#[path = "MetalFX/MetalFX.rs"]
pub mod metal_fx;
#[path = "QuartzCore/QuartzCore.rs"]
pub mod quartz_core;

pub use foundation::{Error, ErrorKind};
pub use metal::{
    BlitCommandEncoder, Buffer, CaptureSession, ClearColor, CommandBuffer, CommandBufferStatus,
    CommandQueue, CompileOptions, CompletionToken, ComputeCommandEncoder, ComputePassDescriptor,
    ComputePipelineState, Device, Function, Library, MAX_TIMESTAMP_COUNTERS, Origin, PixelFormat,
    PrimitiveType, Region, RenderCommandEncoder, RenderPassDescriptor, RenderPipelineDescriptor,
    RenderPipelineState, ResourceOptions, Size, StorageMode, Texture, TextureDescriptor,
    TextureType, TextureUsage, TimestampCounterHeap, Viewport,
};
pub use metal_fx::{
    SpatialScaler, SpatialScalerDescriptor, TemporalScaler, TemporalScalerDescriptor,
};
pub use quartz_core::{Drawable, Layer};
