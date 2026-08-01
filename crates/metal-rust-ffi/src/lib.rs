#![allow(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]

//! Audited Objective-C interoperability boundary for `metal-rust`.
//!
//! This crate is published only because Cargo registries resolve workspace
//! dependencies as separate packages. Applications should depend on
//! `metal-rust`. Its directory and file layout follows metal-cpp while
//! converting generated Objective-C bindings into the narrow safe interface
//! consumed by the public crate. No raw Objective-C binding is re-exported.

#[path = "Foundation/Foundation.rs"]
mod foundation;
#[path = "Metal/Metal.rs"]
mod metal;
#[path = "MetalFX/MetalFX.rs"]
mod metal_fx;

pub use foundation::Error;
pub use metal::{
    BlitCommandEncoder, Buffer, CaptureSession, ClearColor, CommandBuffer, CommandBufferStatus,
    CommandQueue, CompileOptions, ComputeCommandEncoder, ComputePassDescriptor,
    ComputePipelineState, Device, Function, Library, MAX_TIMESTAMP_COUNTERS, Origin, PixelFormat,
    PrimitiveType, Region, RenderCommandEncoder, RenderPassDescriptor, RenderPipelineDescriptor,
    RenderPipelineState, ResourceOptions, Size, StorageMode, Texture, TextureDescriptor,
    TextureType, TextureUsage, TimestampCounterHeap, Viewport,
};
pub use metal_fx::{
    SpatialScaler, SpatialScalerDescriptor, TemporalScaler, TemporalScalerDescriptor,
};
