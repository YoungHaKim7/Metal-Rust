//! Safe Foundation value types used by the Metal facade.

#[path = "NSError.rs"]
mod ns_error;

pub use ns_error::{Error, ErrorKind};
