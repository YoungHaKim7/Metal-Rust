use crate::foundation::{Error, metal_error};
use crate::metal::Device;
use objc2::rc::Retained;
use objc2_foundation::{NSString, NSURL};
use objc2_metal::{MTLCaptureDescriptor, MTLCaptureDestination, MTLCaptureManager};

/// An active device-wide Metal capture session.
pub struct CaptureSession {
    manager: Retained<MTLCaptureManager>,
    active: bool,
}

impl CaptureSession {
    /// Starts a GPU trace document capture at an absolute path.
    pub fn start(device: &Device, output_path: &str) -> Result<Self, Error> {
        // SAFETY: MTLCaptureManager is a process singleton. objc2 marks this
        // accessor unsafe because the generated ownership annotation cannot
        // express that contract; the retained return owns its reference.
        let manager = unsafe { MTLCaptureManager::sharedCaptureManager() };
        if !manager.supportsDestination(MTLCaptureDestination::GPUTraceDocument) {
            return Err(Error::unsupported(
                "GPU trace document capture is not supported",
            ));
        }

        let descriptor = MTLCaptureDescriptor::new();
        descriptor.set_capture_device(&device.inner);
        descriptor.setDestination(MTLCaptureDestination::GPUTraceDocument);
        let path = NSString::from_str(output_path);
        let url = NSURL::fileURLWithPath(&path);
        descriptor.setOutputURL(Some(&url));
        manager
            .startCaptureWithDescriptor_error(&descriptor)
            .map_err(|error| metal_error(&error))?;
        Ok(Self {
            manager,
            active: true,
        })
    }

    /// Stops capture and consumes the session.
    pub fn finish(mut self) {
        self.stop();
    }

    fn stop(&mut self) {
        if self.active {
            self.manager.stopCapture();
            self.active = false;
        }
    }
}

impl Drop for CaptureSession {
    fn drop(&mut self) {
        self.stop();
    }
}
