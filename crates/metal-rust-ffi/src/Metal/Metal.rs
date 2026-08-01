#[path = "MTL4Counters.rs"]
mod mtl4_counters;
#[path = "MTLBlitCommandEncoder.rs"]
mod mtl_blit_command_encoder;
#[path = "MTLBuffer.rs"]
mod mtl_buffer;
#[path = "MTLCaptureManager.rs"]
mod mtl_capture_manager;
#[path = "MTLCommandBuffer.rs"]
mod mtl_command_buffer;
#[path = "MTLCommandEncoder.rs"]
mod mtl_command_encoder;
#[path = "MTLCommandQueue.rs"]
mod mtl_command_queue;
#[path = "MTLDevice.rs"]
mod mtl_device;
#[path = "MTLRenderPass.rs"]
mod mtl_render_pass;
#[path = "MTLShader.rs"]
mod mtl_shader;
#[path = "MTLTexture.rs"]
mod mtl_texture;
#[path = "MTLTypes.rs"]
mod mtl_types;

pub use mtl_blit_command_encoder::BlitCommandEncoder;
pub use mtl_buffer::Buffer;
pub use mtl_capture_manager::CaptureSession;
pub use mtl_command_buffer::CommandBuffer;
pub use mtl_command_encoder::{ComputeCommandEncoder, RenderCommandEncoder};
pub use mtl_command_queue::CommandQueue;
pub use mtl_device::Device;
pub use mtl_render_pass::{ComputePassDescriptor, RenderPassDescriptor};
pub use mtl_shader::{
    CompileOptions, ComputePipelineState, Function, Library, RenderPipelineDescriptor,
    RenderPipelineState,
};
pub use mtl_texture::{Texture, TextureDescriptor};
pub use mtl_types::{
    ClearColor, CommandBufferStatus, Origin, PixelFormat, PrimitiveType, Region, ResourceOptions,
    Size, StorageMode, TextureType, TextureUsage, Viewport,
};
pub use mtl4_counters::{MAX_TIMESTAMP_COUNTERS, TimestampCounterHeap};
