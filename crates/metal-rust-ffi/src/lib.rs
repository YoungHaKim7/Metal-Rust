#![allow(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

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

pub use foundation::Error;
pub use metal::{CaptureSession, Device, MAX_TIMESTAMP_COUNTERS, TimestampCounterHeap};
