use crate::foundation::Error;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_foundation::NSRange;
use objc2_metal::MTL4CounterHeap;
use std::ops::Range;

pub const MAX_TIMESTAMP_COUNTERS: usize = 4096;

pub struct TimestampCounterHeap {
    inner: Retained<ProtocolObject<dyn MTL4CounterHeap>>,
    count: usize,
}

impl TimestampCounterHeap {
    pub(super) const fn new(
        inner: Retained<ProtocolObject<dyn MTL4CounterHeap>>,
        count: usize,
    ) -> Self {
        Self { inner, count }
    }

    #[must_use]
    pub const fn count(&self) -> usize {
        self.count
    }

    pub fn resolve(&self, range: Range<usize>) -> Result<Vec<u64>, Error> {
        if range.start > range.end || range.end > self.count {
            return Err(Error::unsupported("counter resolve range is out of bounds"));
        }
        let length = range.end - range.start;
        if length == 0 {
            return Ok(Vec::new());
        }
        // SAFETY: The range was checked against the immutable heap count. Metal
        // owns the returned NSData and objc2 copies it into a retained object.
        let data = unsafe {
            self.inner
                .resolveCounterRange(NSRange::new(range.start, length))
        }
        .ok_or_else(|| Error::unsupported("Metal returned no counter data"))?;
        let bytes = data.to_vec();
        let expected = length
            .checked_mul(size_of::<u64>())
            .ok_or_else(|| Error::unsupported("counter result size overflow"))?;
        if bytes.len() != expected {
            return Err(Error::unsupported(format!(
                "Metal returned {} counter bytes, expected {expected}",
                bytes.len()
            )));
        }
        Ok(bytes
            .chunks_exact(size_of::<u64>())
            .map(|chunk| {
                let mut value = [0_u8; size_of::<u64>()];
                value.copy_from_slice(chunk);
                u64::from_ne_bytes(value)
            })
            .collect())
    }
}
