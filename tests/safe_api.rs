#![forbid(unsafe_code)]

use metal_rust::{Device, ErrorKind, MAX_TIMESTAMP_COUNTERS};

#[test]
fn default_device_has_an_owned_name() {
    let Some(device) = Device::system_default() else {
        return;
    };
    assert!(!device.name().is_empty());
}

#[test]
fn counter_count_is_checked_before_metal() {
    let Some(device) = Device::system_default() else {
        return;
    };
    let zero = device.new_timestamp_counter_heap(0).err().unwrap();
    assert_eq!(zero.kind(), ErrorKind::InvalidArgument);
    let too_large = device
        .new_timestamp_counter_heap(MAX_TIMESTAMP_COUNTERS + 1)
        .err()
        .unwrap();
    assert_eq!(too_large.kind(), ErrorKind::InvalidArgument);
}

#[test]
fn metal4_counter_heap_is_safe_when_available() {
    let Some(device) = Device::system_default() else {
        return;
    };
    match device.new_timestamp_counter_heap(2) {
        Ok(heap) => assert_eq!(heap.count(), 2),
        Err(error) if error.kind() == ErrorKind::Unsupported => {}
        Err(error) => panic!("unexpected Metal error: {error}"),
    }
}
