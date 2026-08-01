#[path = "MTL4Counters.rs"]
mod mtl4_counters;
#[path = "MTLCaptureManager.rs"]
mod mtl_capture_manager;
#[path = "MTLDevice.rs"]
mod mtl_device;

pub use mtl_capture_manager::CaptureSession;
pub use mtl_device::Device;
pub use mtl4_counters::{MAX_TIMESTAMP_COUNTERS, TimestampCounterHeap};
