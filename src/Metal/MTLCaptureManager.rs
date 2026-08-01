use crate::{Device, Error};
use std::path::Path;

/// An active device-wide Metal capture.
///
/// Dropping the session stops capture. Call [`finish`](Self::finish) when an
/// explicit frame boundary is available.
pub struct CaptureSession {
    inner: metal_rust_ffi::CaptureSession,
}

impl CaptureSession {
    /// Starts a device-wide GPU trace capture at an absolute output path.
    pub fn start(device: &Device, output_path: &Path) -> Result<Self, Error> {
        if !output_path.is_absolute() {
            return Err(Error::invalid_argument(
                "capture output path must be absolute",
            ));
        }
        let path = output_path
            .to_str()
            .ok_or_else(|| Error::invalid_argument("capture output path must be UTF-8"))?;
        if path.as_bytes().contains(&0) {
            return Err(Error::invalid_argument("capture output path contains NUL"));
        }
        metal_rust_ffi::CaptureSession::start(&device.inner, path)
            .map(|inner| Self { inner })
            .map_err(Error::from_ffi)
    }

    /// Stops capture immediately and consumes the session.
    pub fn finish(self) {
        self.inner.finish();
    }
}
