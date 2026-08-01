use super::CompletionToken;
use crate::Error;
use std::ops::Range;

/// Maximum timestamp entries accepted by the safe Metal 4 API.
pub const MAX_TIMESTAMP_COUNTERS: usize = metal_rust_ffi::MAX_TIMESTAMP_COUNTERS;

/// An owned Metal 4 timestamp counter heap.
pub struct TimestampCounterHeap {
    inner: metal_rust_ffi::TimestampCounterHeap,
}

impl TimestampCounterHeap {
    pub(crate) const fn from_ffi(inner: metal_rust_ffi::TimestampCounterHeap) -> Self {
        Self { inner }
    }

    /// Returns the number of timestamp entries in the heap.
    #[must_use]
    pub const fn count(&self) -> usize {
        self.inner.count()
    }

    /// Resolves a checked range of GPU timestamps after command completion.
    ///
    /// The caller is responsible only for logical GPU synchronization; no raw
    /// memory or Objective-C lifetime is exposed by this operation.
    pub fn resolve(
        &self,
        range: Range<usize>,
        _completion: &CompletionToken<'_>,
    ) -> Result<Vec<u64>, Error> {
        if range.start > range.end || range.end > self.count() {
            return Err(Error::invalid_argument("counter range is out of bounds"));
        }
        self.inner.resolve(range).map_err(Error::from_ffi)
    }
}
