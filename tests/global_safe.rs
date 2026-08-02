#![forbid(unsafe_code)]

use metal_rust::metal::{
    METAL_CPP_VERSION_MAJOR, METAL_CPP_VERSION_MINOR, METAL_CPP_VERSION_PATCH, PackedFloat3,
    PackedFloat4x3, PackedFloatQuaternion, metal_cpp_supports_version,
};

#[test]
fn packed_float_values_have_safe_construction_and_indexing() {
    let mut vector = PackedFloat3::from_components(1.0, 2.0, 3.0);
    assert_eq!(vector[1], 2.0);
    vector[1] = 4.0;
    assert_eq!(vector.into_array(), [1.0, 4.0, 3.0]);

    let mut matrix = PackedFloat4x3::new([
        PackedFloat3::new([1.0, 0.0, 0.0]),
        PackedFloat3::new([0.0, 1.0, 0.0]),
        PackedFloat3::new([0.0, 0.0, 1.0]),
        PackedFloat3::new([4.0, 5.0, 6.0]),
    ]);
    matrix[3][0] = 7.0;
    assert_eq!(matrix.columns[3].into_array(), [7.0, 5.0, 6.0]);

    let quaternion = PackedFloatQuaternion::from_components(0.0, 0.0, 0.0, 1.0);
    assert_eq!(quaternion[3], 1.0);
}

#[test]
fn reference_version_query_is_lexicographic() {
    assert_eq!(METAL_CPP_VERSION_MAJOR, 381);
    assert_eq!(METAL_CPP_VERSION_MINOR, 0);
    assert_eq!(METAL_CPP_VERSION_PATCH, 0);
    assert!(metal_cpp_supports_version(381, 0, 0));
    assert!(metal_cpp_supports_version(380, u32::MAX, u32::MAX));
    assert!(!metal_cpp_supports_version(381, 0, 1));
    assert!(!metal_cpp_supports_version(382, 0, 0));
}
