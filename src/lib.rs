#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Safe Rust interfaces for Apple Metal.
//!
//! Raw Objective-C objects and unsafe operations are confined to the audited
//! `metal-rust-ffi` implementation crate. This public crate never exposes
//! object pointers or requires callers to uphold Objective-C runtime
//! invariants.

#[path = "Foundation/Foundation.rs"]
pub mod foundation;
#[path = "Metal/Metal.rs"]
pub mod metal;
#[path = "MetalFX/MetalFX.rs"]
pub mod metal_fx;
#[path = "QuartzCore/QuartzCore.rs"]
pub mod quartz_core;

pub use foundation::{Error, ErrorKind};
pub use metal::{CaptureSession, Device, MAX_TIMESTAMP_COUNTERS, TimestampCounterHeap};
