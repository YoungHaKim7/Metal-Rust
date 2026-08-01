//! Audited MetalFX temporal scaler binding.
#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)]

use crate::foundation::Error;
use crate::metal::{CommandBuffer, Device, PixelFormat, StorageMode, Texture};
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, ProtocolObject};
use objc2::{extern_class, extern_conformance, extern_methods, extern_protocol};
use objc2_foundation::{NSCopying, NSObject, NSObjectProtocol};
use objc2_metal::{MTLCommandBuffer, MTLDevice, MTLTexture, MTLTextureUsage};

extern_protocol!(
    /// MetalFX temporal scaler protocol declared by the macOS SDK.
    ///
    /// # Safety
    ///
    /// This trait is implemented only by the Objective-C runtime object whose
    /// selectors and return layouts match the SDK protocol declaration.
    pub unsafe trait MTLFXTemporalScaler: NSObjectProtocol {
        #[unsafe(method(encodeToCommandBuffer:))]
        #[unsafe(method_family = none)]
        fn encodeToCommandBuffer(&self, command_buffer: &ProtocolObject<dyn MTLCommandBuffer>);

        #[unsafe(method(colorTextureUsage))]
        #[unsafe(method_family = none)]
        fn colorTextureUsage(&self) -> MTLTextureUsage;

        #[unsafe(method(depthTextureUsage))]
        #[unsafe(method_family = none)]
        fn depthTextureUsage(&self) -> MTLTextureUsage;

        #[unsafe(method(motionTextureUsage))]
        #[unsafe(method_family = none)]
        fn motionTextureUsage(&self) -> MTLTextureUsage;

        #[unsafe(method(outputTextureUsage))]
        #[unsafe(method_family = none)]
        fn outputTextureUsage(&self) -> MTLTextureUsage;

        #[unsafe(method(setInputContentWidth:))]
        #[unsafe(method_family = none)]
        fn setInputContentWidth(&self, width: usize);

        #[unsafe(method(setInputContentHeight:))]
        #[unsafe(method_family = none)]
        fn setInputContentHeight(&self, height: usize);

        #[unsafe(method(setColorTexture:))]
        #[unsafe(method_family = none)]
        fn setColorTexture(&self, texture: Option<&ProtocolObject<dyn MTLTexture>>);

        #[unsafe(method(setDepthTexture:))]
        #[unsafe(method_family = none)]
        fn setDepthTexture(&self, texture: Option<&ProtocolObject<dyn MTLTexture>>);

        #[unsafe(method(setMotionTexture:))]
        #[unsafe(method_family = none)]
        fn setMotionTexture(&self, texture: Option<&ProtocolObject<dyn MTLTexture>>);

        #[unsafe(method(setOutputTexture:))]
        #[unsafe(method_family = none)]
        fn setOutputTexture(&self, texture: Option<&ProtocolObject<dyn MTLTexture>>);
    }
);

extern_class!(
    /// MetalFX temporal scaler descriptor declared by the macOS SDK.
    #[unsafe(super(NSObject))]
    pub struct MTLFXTemporalScalerDescriptor;
);

extern_conformance!(
    unsafe impl NSObjectProtocol for MTLFXTemporalScalerDescriptor {}
);
extern_conformance!(
    unsafe impl NSCopying for MTLFXTemporalScalerDescriptor {}
);

impl MTLFXTemporalScalerDescriptor {
    extern_methods!(
        #[unsafe(method(new))]
        #[unsafe(method_family = new)]
        pub fn new() -> Retained<Self>;

        #[unsafe(method(setColorTextureFormat:))]
        #[unsafe(method_family = none)]
        pub fn setColorTextureFormat(&self, format: objc2_metal::MTLPixelFormat);

        #[unsafe(method(setDepthTextureFormat:))]
        #[unsafe(method_family = none)]
        pub fn setDepthTextureFormat(&self, format: objc2_metal::MTLPixelFormat);

        #[unsafe(method(setMotionTextureFormat:))]
        #[unsafe(method_family = none)]
        pub fn setMotionTextureFormat(&self, format: objc2_metal::MTLPixelFormat);

        #[unsafe(method(setOutputTextureFormat:))]
        #[unsafe(method_family = none)]
        pub fn setOutputTextureFormat(&self, format: objc2_metal::MTLPixelFormat);

        #[unsafe(method(setInputWidth:))]
        #[unsafe(method_family = none)]
        pub fn setInputWidth(&self, width: usize);

        #[unsafe(method(setInputHeight:))]
        #[unsafe(method_family = none)]
        pub fn setInputHeight(&self, height: usize);

        #[unsafe(method(setOutputWidth:))]
        #[unsafe(method_family = none)]
        pub fn setOutputWidth(&self, width: usize);

        #[unsafe(method(setOutputHeight:))]
        #[unsafe(method_family = none)]
        pub fn setOutputHeight(&self, height: usize);

        #[unsafe(method(newTemporalScalerWithDevice:))]
        #[unsafe(method_family = new)]
        pub fn newTemporalScalerWithDevice(
            &self,
            device: &ProtocolObject<dyn MTLDevice>,
        ) -> Option<Retained<ProtocolObject<dyn MTLFXTemporalScaler>>>;

        #[unsafe(method(supportsDevice:))]
        #[unsafe(method_family = none)]
        pub fn supportsDevice(device: &ProtocolObject<dyn MTLDevice>) -> bool;

        #[unsafe(method(supportedInputContentMinScaleForDevice:))]
        #[unsafe(method_family = none)]
        pub fn supportedInputContentMinScaleForDevice(
            device: &ProtocolObject<dyn MTLDevice>,
        ) -> f32;

        #[unsafe(method(supportedInputContentMaxScaleForDevice:))]
        #[unsafe(method_family = none)]
        pub fn supportedInputContentMaxScaleForDevice(
            device: &ProtocolObject<dyn MTLDevice>,
        ) -> f32;
    );
}

/// An owned temporal-scaler descriptor.
pub struct TemporalScalerDescriptor {
    inner: Retained<MTLFXTemporalScalerDescriptor>,
}

impl TemporalScalerDescriptor {
    /// Creates a descriptor when the MetalFX class is present at runtime.
    pub fn new() -> Result<Self, Error> {
        if AnyClass::get(c"MTLFXTemporalScalerDescriptor").is_none() {
            return Err(Error::unsupported(
                "MetalFX temporal scaling is unavailable on this system",
            ));
        }
        Ok(Self {
            inner: MTLFXTemporalScalerDescriptor::new(),
        })
    }

    /// Sets formats and dimensions after validating non-zero dimensions.
    #[allow(clippy::too_many_arguments)]
    pub fn configure(
        &self,
        color_format: PixelFormat,
        depth_format: PixelFormat,
        motion_format: PixelFormat,
        output_format: PixelFormat,
        input_width: usize,
        input_height: usize,
        output_width: usize,
        output_height: usize,
    ) -> Result<(), Error> {
        if input_width == 0 || input_height == 0 || output_width == 0 || output_height == 0 {
            return Err(Error::invalid_argument(
                "MetalFX dimensions must be greater than zero",
            ));
        }
        self.inner.setColorTextureFormat(color_format.as_objc());
        self.inner.setDepthTextureFormat(depth_format.as_objc());
        self.inner.setMotionTextureFormat(motion_format.as_objc());
        self.inner.setOutputTextureFormat(output_format.as_objc());
        self.inner.setInputWidth(input_width);
        self.inner.setInputHeight(input_height);
        self.inner.setOutputWidth(output_width);
        self.inner.setOutputHeight(output_height);
        Ok(())
    }

    /// Returns whether the installed MetalFX runtime supports the device.
    #[must_use]
    pub fn supports_device(&self, device: &Device) -> bool {
        MTLFXTemporalScalerDescriptor::supportsDevice(&device.inner)
    }

    /// Returns the minimum input content scale supported by MetalFX.
    #[must_use]
    pub fn supported_input_content_min_scale(&self, device: &Device) -> f32 {
        MTLFXTemporalScalerDescriptor::supportedInputContentMinScaleForDevice(&device.inner)
    }

    /// Returns the maximum input content scale supported by MetalFX.
    #[must_use]
    pub fn supported_input_content_max_scale(&self, device: &Device) -> f32 {
        MTLFXTemporalScalerDescriptor::supportedInputContentMaxScaleForDevice(&device.inner)
    }

    /// Creates the scaler, or returns a platform capability error.
    pub fn new_scaler(&self, device: &Device) -> Result<TemporalScaler, Error> {
        self.inner
            .newTemporalScalerWithDevice(&device.inner)
            .map(|inner| TemporalScaler { inner })
            .ok_or_else(|| Error::unsupported("MetalFX temporal scaling is unavailable"))
    }
}

/// An owned MetalFX temporal scaler.
pub struct TemporalScaler {
    inner: Retained<ProtocolObject<dyn MTLFXTemporalScaler>>,
}

impl TemporalScaler {
    /// Encodes the temporal scaling operation into a command buffer.
    pub fn encode(&self, command_buffer: &CommandBuffer) {
        self.inner.encodeToCommandBuffer(&command_buffer.inner);
    }

    /// Returns the input texture usage required by the scaler.
    #[must_use]
    pub fn color_texture_usage(&self) -> u64 {
        self.inner.colorTextureUsage().0 as u64
    }

    /// Returns the output texture usage required by the scaler.
    #[must_use]
    pub fn output_texture_usage(&self) -> u64 {
        self.inner.outputTextureUsage().0 as u64
    }

    /// Returns the required input depth texture usage bitmask.
    #[must_use]
    pub fn depth_texture_usage(&self) -> u64 {
        self.inner.depthTextureUsage().0 as u64
    }

    /// Returns the required input motion texture usage bitmask.
    #[must_use]
    pub fn motion_texture_usage(&self) -> u64 {
        self.inner.motionTextureUsage().0 as u64
    }

    /// Sets the input content dimensions after checking they are non-zero.
    pub fn set_input_content_size(&self, width: usize, height: usize) -> Result<(), Error> {
        if width == 0 || height == 0 {
            return Err(Error::invalid_argument(
                "MetalFX input content dimensions must be greater than zero",
            ));
        }
        self.inner.setInputContentWidth(width);
        self.inner.setInputContentHeight(height);
        Ok(())
    }

    /// Assigns the color, depth, motion, and output textures. The output must
    /// use private storage.
    pub fn set_textures(
        &self,
        color: Option<&Texture>,
        depth: Option<&Texture>,
        motion: Option<&Texture>,
        output: Option<&Texture>,
    ) -> Result<(), Error> {
        if let Some(texture) = output
            && texture.storage_mode() != StorageMode::Private
        {
            return Err(Error::invalid_argument(
                "MetalFX output texture must use private storage",
            ));
        }
        self.inner
            .setColorTexture(color.map(|texture| &*texture.inner));
        self.inner
            .setDepthTexture(depth.map(|texture| &*texture.inner));
        self.inner
            .setMotionTexture(motion.map(|texture| &*texture.inner));
        self.inner
            .setOutputTexture(output.map(|texture| &*texture.inner));
        Ok(())
    }
}
