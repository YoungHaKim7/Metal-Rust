#![forbid(unsafe_code)]

use metal_rust::{
    ClearColor, Device, ErrorKind, Layer, MAX_TIMESTAMP_COUNTERS, Origin, PixelFormat, Region,
    ResourceOptions, Size, SpatialScalerDescriptor, TemporalScalerDescriptor, TextureDescriptor,
    TextureUsage,
};

#[test]
fn checked_value_constructors_are_stable() {
    let region = Region::new(Origin::new(1, 2, 0), Size::new(8, 4, 1));
    assert_eq!(region.size.width, 8);
    assert_ne!(
        TextureUsage::SHADER_READ.union(TextureUsage::RENDER_TARGET),
        TextureUsage::UNKNOWN
    );
    assert_eq!(
        TextureUsage::SHADER_READ.union(TextureUsage::UNKNOWN),
        TextureUsage::SHADER_READ
    );
    assert_eq!(ResourceOptions::SHARED, ResourceOptions::default());
    assert_eq!(ClearColor::new(0.1, 0.2, 0.3, 1.0).alpha, 1.0);
}

#[test]
fn texture_descriptor_rejects_invalid_dimensions_and_mips() {
    let zero_width = TextureDescriptor::new_2d(
        PixelFormat::RGBA8_UNORM,
        0,
        1,
        ResourceOptions::SHARED,
        TextureUsage::SHADER_READ,
    )
    .err()
    .expect("zero width must be rejected before Metal");
    assert_eq!(zero_width.kind(), ErrorKind::InvalidArgument);

    let descriptor = TextureDescriptor::new_2d(
        PixelFormat::RGBA8_UNORM,
        1,
        1,
        ResourceOptions::SHARED,
        TextureUsage::SHADER_READ,
    )
    .expect("one-by-one descriptor is valid");
    let zero_mips = match descriptor.set_mipmap_level_count(0) {
        Ok(()) => panic!("zero mip levels must be rejected before Metal"),
        Err(error) => error,
    };
    assert_eq!(zero_mips.kind(), ErrorKind::InvalidArgument);
}

#[test]
fn metalfx_descriptors_reject_zero_dimensions_without_runtime_creation() {
    let spatial = match SpatialScalerDescriptor::new() {
        Ok(descriptor) => descriptor,
        Err(error) if error.kind() == ErrorKind::Unsupported => return,
        Err(error) => panic!("unexpected spatial descriptor error: {error}"),
    };
    let spatial_error = match spatial.configure(
        PixelFormat::RGBA8_UNORM,
        PixelFormat::RGBA8_UNORM,
        0,
        1080,
        1920,
        1080,
    ) {
        Ok(()) => panic!("spatial zero width must be rejected"),
        Err(error) => error,
    };
    assert_eq!(spatial_error.kind(), ErrorKind::InvalidArgument);

    let temporal = match TemporalScalerDescriptor::new() {
        Ok(descriptor) => descriptor,
        Err(error) if error.kind() == ErrorKind::Unsupported => return,
        Err(error) => panic!("unexpected temporal descriptor error: {error}"),
    };
    let temporal_error = match temporal.configure(
        PixelFormat::RGBA8_UNORM,
        PixelFormat::RGBA8_UNORM,
        PixelFormat::RGBA8_UNORM,
        PixelFormat::RGBA8_UNORM,
        1920,
        1080,
        0,
        1080,
    ) {
        Ok(()) => panic!("temporal zero width must be rejected"),
        Err(error) => error,
    };
    assert_eq!(temporal_error.kind(), ErrorKind::InvalidArgument);
}

#[test]
fn metalfx_scaler_creation_is_runtime_gated() {
    let Some(device) = Device::system_default() else {
        return;
    };
    let spatial = match SpatialScalerDescriptor::new() {
        Ok(descriptor) => descriptor,
        Err(error) if error.kind() == ErrorKind::Unsupported => return,
        Err(error) => panic!("unexpected spatial descriptor error: {error}"),
    };
    spatial
        .configure(
            PixelFormat::BGRA8_UNORM,
            PixelFormat::BGRA8_UNORM,
            1920,
            1080,
            3840,
            2160,
        )
        .expect("valid spatial configuration");
    if spatial.supports_device(&device) {
        let scaler = match spatial.new_scaler(&device) {
            Ok(scaler) => scaler,
            Err(error) if error.kind() == ErrorKind::Unsupported => return,
            Err(error) => panic!("unexpected spatial scaler error: {error}"),
        };
        assert_ne!(scaler.output_texture_usage(), 0);
        scaler
            .set_input_content_size(1920, 1080)
            .expect("valid spatial content size");
    }

    let temporal = match TemporalScalerDescriptor::new() {
        Ok(descriptor) => descriptor,
        Err(error) if error.kind() == ErrorKind::Unsupported => return,
        Err(error) => panic!("unexpected temporal descriptor error: {error}"),
    };
    temporal
        .configure(
            PixelFormat::BGRA8_UNORM,
            PixelFormat::DEPTH32_FLOAT,
            PixelFormat::RG16_FLOAT,
            PixelFormat::BGRA8_UNORM,
            1920,
            1080,
            3840,
            2160,
        )
        .expect("valid temporal configuration");
    if temporal.supports_device(&device) {
        let scaler = match temporal.new_scaler(&device) {
            Ok(scaler) => scaler,
            Err(error) if error.kind() == ErrorKind::Unsupported => return,
            Err(error) => panic!("unexpected temporal scaler error: {error}"),
        };
        assert_ne!(scaler.output_texture_usage(), 0);
        scaler
            .set_input_content_size(1920, 1080)
            .expect("valid temporal content size");
    }
}

#[test]
fn metal_layer_is_owned_and_configurable_without_a_device() {
    let layer = Layer::new();
    layer.set_pixel_format(PixelFormat::BGRA8_UNORM);
    assert!(layer.device().is_none());
}

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

#[test]
fn core_submission_smoke_when_available() {
    let Some(device) = Device::system_default() else {
        return;
    };
    let queue = match device.new_command_queue(Some(2)) {
        Ok(queue) => queue,
        Err(error) if error.kind() == ErrorKind::Unsupported => return,
        Err(error) => panic!("unexpected queue error: {error}"),
    };
    queue.set_label(Some("metal-rust-safe-api-test"));
    assert_eq!(queue.label().as_deref(), Some("metal-rust-safe-api-test"));

    let buffer = match device.new_buffer(64, ResourceOptions::SHARED) {
        Ok(buffer) => buffer,
        Err(error) if error.kind() == ErrorKind::Unsupported => return,
        Err(error) => panic!("unexpected buffer error: {error}"),
    };
    assert_eq!(buffer.length(), 64);
    buffer
        .write(4, &[1, 2, 3, 4])
        .expect("shared buffer is writable");
    let range_error = match buffer.write(63, &[1, 2]) {
        Ok(()) => panic!("buffer range must be checked"),
        Err(error) => error,
    };
    assert_eq!(range_error.kind(), ErrorKind::InvalidArgument);

    let destination = match device.new_buffer(64, ResourceOptions::SHARED) {
        Ok(buffer) => buffer,
        Err(error) if error.kind() == ErrorKind::Unsupported => return,
        Err(error) => panic!("unexpected destination buffer error: {error}"),
    };
    let mut command_buffer = match queue.command_buffer() {
        Ok(command_buffer) => command_buffer,
        Err(error) if error.kind() == ErrorKind::Unsupported => return,
        Err(error) => panic!("unexpected command buffer error: {error}"),
    };
    {
        let blit = match command_buffer.blit_encoder() {
            Ok(blit) => blit,
            Err(error) if error.kind() == ErrorKind::Unsupported => return,
            Err(error) => panic!("unexpected blit encoder error: {error}"),
        };
        blit.copy_buffer(&buffer, 4, &destination, 8, 4)
            .expect("checked shared-buffer copy must encode");
        blit.end_encoding();
    }
    command_buffer.commit();
    let completion = command_buffer.wait_until_completed();
    assert_eq!(
        command_buffer.status(),
        metal_rust::CommandBufferStatus::Completed,
        "command buffer failed: {:?}",
        command_buffer.error()
    );
    assert_eq!(
        destination
            .read(8, 4, &completion)
            .expect("shared readback is valid"),
        [1, 2, 3, 4]
    );
}

#[test]
fn shader_pipeline_and_render_compute_encoders_smoke_when_available() {
    let Some(device) = Device::system_default() else {
        return;
    };
    let source = r#"
        #include <metal_stdlib>
        using namespace metal;

        kernel void compute_main(device uint *data [[buffer(0)]],
                                 uint id [[thread_position_in_grid]]) {
            if (id == 0) {
                data[0] = 42;
            }
        }

        struct VertexOut {
            float4 position [[position]];
        };

        vertex VertexOut vertex_main(uint vertex_id [[vertex_id]]) {
            constexpr float2 positions[3] = {
                float2(-1.0, -1.0), float2(3.0, -1.0), float2(-1.0, 3.0)
            };
            return VertexOut{float4(positions[vertex_id], 0.0, 1.0)};
        }

        fragment float4 fragment_main() {
            return float4(1.0, 0.0, 0.0, 1.0);
        }
    "#;
    let library =
        match device.new_library_from_source(source, Some(&metal_rust::CompileOptions::strict())) {
            Ok(library) => library,
            Err(error) if error.kind() == ErrorKind::Unsupported => return,
            Err(error) => panic!("unexpected shader compilation error: {error}"),
        };
    let compute_function = library
        .function("compute_main")
        .expect("compute function exists");
    let vertex_function = library
        .function("vertex_main")
        .expect("vertex function exists");
    let fragment_function = library
        .function("fragment_main")
        .expect("fragment function exists");
    let compute_pipeline = match device.new_compute_pipeline(&compute_function) {
        Ok(pipeline) => pipeline,
        Err(error) if error.kind() == ErrorKind::Unsupported => return,
        Err(error) => panic!("unexpected compute pipeline error: {error}"),
    };
    let render_pipeline =
        match device.new_render_pipeline(&metal_rust::RenderPipelineDescriptor::new(
            &vertex_function,
            Some(&fragment_function),
            PixelFormat::BGRA8_UNORM,
        )) {
            Ok(pipeline) => pipeline,
            Err(error) if error.kind() == ErrorKind::Unsupported => return,
            Err(error) => panic!("unexpected render pipeline error: {error}"),
        };
    assert!(compute_pipeline.max_total_threads_per_threadgroup() > 0);

    let texture_descriptor = TextureDescriptor::new_2d(
        PixelFormat::BGRA8_UNORM,
        4,
        4,
        ResourceOptions::SHARED,
        TextureUsage::RENDER_TARGET,
    )
    .expect("render texture descriptor is valid");
    let texture = match device.new_texture(&texture_descriptor) {
        Ok(texture) => texture,
        Err(error) if error.kind() == ErrorKind::Unsupported => return,
        Err(error) => panic!("unexpected texture error: {error}"),
    };
    let buffer = match device.new_buffer(16, ResourceOptions::SHARED) {
        Ok(buffer) => buffer,
        Err(error) if error.kind() == ErrorKind::Unsupported => return,
        Err(error) => panic!("unexpected shader buffer error: {error}"),
    };
    let queue = match device.new_command_queue(None) {
        Ok(queue) => queue,
        Err(error) if error.kind() == ErrorKind::Unsupported => return,
        Err(error) => panic!("unexpected shader queue error: {error}"),
    };
    let mut command_buffer = match queue.command_buffer() {
        Ok(command_buffer) => command_buffer,
        Err(error) if error.kind() == ErrorKind::Unsupported => return,
        Err(error) => panic!("unexpected shader command buffer error: {error}"),
    };
    {
        let encoder = command_buffer
            .compute_encoder(&metal_rust::ComputePassDescriptor::new())
            .expect("compute encoder exists");
        encoder.set_pipeline(&compute_pipeline);
        encoder
            .set_buffer(&buffer, 0, 0)
            .expect("compute buffer binding is valid");
        encoder
            .dispatch_1d(1, 1)
            .expect("one-thread dispatch is valid");
        encoder.end_encoding();
    }
    {
        let pass = metal_rust::RenderPassDescriptor::new();
        pass.set_color_attachment(0, &texture, ClearColor::new(0.0, 0.0, 0.0, 1.0))
            .expect("color attachment index is valid");
        let encoder = command_buffer
            .render_encoder(&pass)
            .expect("render encoder exists");
        encoder.set_pipeline(&render_pipeline);
        encoder
            .set_viewport(metal_rust::Viewport::new(0.0, 0.0, 4.0, 4.0, 0.0, 1.0))
            .expect("finite viewport is valid");
        encoder
            .draw_primitives(metal_rust::PrimitiveType::Triangle, 0, 3)
            .expect("triangle draw is valid");
        encoder.end_encoding();
    }
    command_buffer.commit();
    let completion = command_buffer.wait_until_completed();
    assert_eq!(
        command_buffer.status(),
        metal_rust::CommandBufferStatus::Completed,
        "shader command buffer failed: {:?}",
        command_buffer.error()
    );
    assert_eq!(
        buffer
            .read(0, 4, &completion)
            .expect("shared readback is valid"),
        42_u32.to_ne_bytes()
    );
}

#[test]
fn drawable_and_capture_paths_are_runtime_gated() {
    let Some(device) = Device::system_default() else {
        return;
    };
    let layer = Layer::with_device(&device);
    layer.set_pixel_format(PixelFormat::BGRA8_UNORM);
    if let Ok(drawable) = layer.next_drawable() {
        let queue = match device.new_command_queue(None) {
            Ok(queue) => queue,
            Err(error) if error.kind() == ErrorKind::Unsupported => return,
            Err(error) => panic!("unexpected drawable queue error: {error}"),
        };
        let mut command_buffer = queue.command_buffer().expect("drawable command buffer");
        command_buffer.present_drawable(&drawable);
        command_buffer.commit();
        command_buffer.wait_until_completed();
    }

    let path = std::env::temp_dir().join(format!(
        "metal-rust-safe-api-{}.gputrace",
        std::process::id()
    ));
    match metal_rust::CaptureSession::start(&device, &path) {
        Ok(session) => session.finish(),
        Err(error) if error.kind() == ErrorKind::Unsupported => {}
        Err(error) => panic!("unexpected capture error: {error}"),
    }
    let _ = std::fs::remove_file(path);
}
