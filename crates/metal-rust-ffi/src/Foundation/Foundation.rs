#[path = "NSError.rs"]
mod ns_error;

pub use ns_error::Error;
pub(crate) use ns_error::metal_error;
