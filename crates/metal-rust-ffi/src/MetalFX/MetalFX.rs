//! Audited MetalFX Objective-C boundary.

#[path = "MTLFXSpatialScaler.rs"]
mod mtl_fx_spatial_scaler;
#[path = "MTLFXTemporalScaler.rs"]
mod mtl_fx_temporal_scaler;

pub use mtl_fx_spatial_scaler::{SpatialScaler, SpatialScalerDescriptor};
pub use mtl_fx_temporal_scaler::{TemporalScaler, TemporalScalerDescriptor};
