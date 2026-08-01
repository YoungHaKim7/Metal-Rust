use crate::foundation::Error;
use crate::metal::{MAX_TIMESTAMP_COUNTERS, TimestampCounterHeap};
use objc2::rc::Retained;
use objc2::runtime::{NSObjectProtocol, ProtocolObject};
use objc2::sel;
use objc2_metal::{
    MTL4CounterHeapDescriptor, MTL4CounterHeapType, MTLCreateSystemDefaultDevice, MTLDevice,
};

#[derive(Clone)]
pub struct Device {
    pub(super) inner: Retained<ProtocolObject<dyn MTLDevice>>,
}

impl Device {
    #[must_use]
    pub fn system_default() -> Option<Self> {
        MTLCreateSystemDefaultDevice().map(|inner| Self { inner })
    }

    #[must_use]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    pub fn new_timestamp_counter_heap(&self, count: usize) -> Result<TimestampCounterHeap, Error> {
        if !(1..=MAX_TIMESTAMP_COUNTERS).contains(&count) {
            return Err(Error::unsupported(format!(
                "timestamp counter count must be between 1 and {MAX_TIMESTAMP_COUNTERS}"
            )));
        }
        if !self
            .inner
            .respondsToSelector(sel!(newCounterHeapWithDescriptor:error:))
        {
            return Err(Error::unsupported(
                "MTL4CounterHeap requires macOS 26 or newer",
            ));
        }

        let descriptor = MTL4CounterHeapDescriptor::new();
        descriptor.setType(MTL4CounterHeapType::Timestamp);
        // SAFETY: The public facade restricts count to the range validated by
        // Metal's current timestamp CounterHeap implementation.
        unsafe { descriptor.setCount(count) };
        self.inner
            .newCounterHeapWithDescriptor_error(&descriptor)
            .map(|inner| TimestampCounterHeap::new(inner, count))
            .map_err(|error| crate::foundation::metal_error(&error))
    }
}
