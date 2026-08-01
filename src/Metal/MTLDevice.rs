use crate::{Error, TimestampCounterHeap};

/// An owned reference to an Apple Metal device.
#[derive(Clone)]
pub struct Device {
    pub(crate) inner: metal_rust_ffi::Device,
}

impl Device {
    /// Returns the preferred system Metal device, if one is available.
    #[must_use]
    pub fn system_default() -> Option<Self> {
        metal_rust_ffi::Device::system_default().map(|inner| Self { inner })
    }

    /// Returns the device's user-visible name as an owned Rust string.
    #[must_use]
    pub fn name(&self) -> String {
        self.inner.name()
    }

    /// Creates a Metal 4 timestamp counter heap.
    ///
    /// The count is validated before the Objective-C boundary is entered.
    pub fn new_timestamp_counter_heap(&self, count: usize) -> Result<TimestampCounterHeap, Error> {
        if count == 0 || count > crate::MAX_TIMESTAMP_COUNTERS {
            return Err(Error::invalid_argument(format!(
                "timestamp counter count must be between 1 and {}",
                crate::MAX_TIMESTAMP_COUNTERS
            )));
        }
        self.inner
            .new_timestamp_counter_heap(count)
            .map(TimestampCounterHeap::from_ffi)
            .map_err(Error::from_ffi)
    }
}
