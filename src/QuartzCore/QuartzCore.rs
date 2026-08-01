//! Safe interfaces corresponding to Apple's QuartzCore framework.
//!
//! `CAMetalLayer` and drawable ownership are represented by RAII wrappers.

#[path = "CAMetalLayer.rs"]
mod ca_metal_layer;

pub use ca_metal_layer::{Drawable, Layer};
