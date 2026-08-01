# API coverage

This file is generated from `api/metal-cpp-inventory.json` and
`api/coverage.json`. The ledger has one explicit entry for every public
declaration in the local Apple metal-cpp snapshot.

Reference digest: `925aa6c677b28fb9fe1a54a217bea1efd3f145c57e0e49e5b82d9d47810ff43d`.

Statuses:

- `implemented`: a pinned/generated or handwritten audited FFI mapping exists; the public facade may group several C++ overloads into one checked Rust operation.
- `availability`: the API is represented, but object creation is gated by
  the Apple framework/runtime capability query.
- `substitute`: C++ ownership, helper, scalar, or pointer shape is
  grouped into a safe Rust value or RAII abstraction.

Generated Metal bindings are implementation sources behind
`metal-rust-ffi`; they are not re-exported as raw Objective-C APIs.
The `rust` target in the ledger preserves selector-level traceability,
while the evidence and notes identify the safe facade grouping or
substitution that callers actually use.

Total declarations: **5226**.
Implemented: **3171**; availability-gated: **397**; substituted: **1658**.

| Framework | Implemented | Availability | Substitute | Total |
| --- | ---: | ---: | ---: | ---: |
| Foundation | 0 | 0 | 302 | 302 |
| Metal | 3156 | 0 | 1339 | 4495 |
| MetalFX | 10 | 397 | 0 | 407 |
| QuartzCore | 5 | 0 | 17 | 22 |

## Header summary

Each reference header is retained in the ledger so a new or removed
header cannot disappear behind a framework-level percentage.

| Framework | Header | Implemented | Availability | Substitute | Total |
| --- | --- | ---: | ---: | ---: | ---: |
| Foundation | `Foundation/NSArray.hpp` | 0 | 0 | 9 | 9 |
| Foundation | `Foundation/NSAutoreleasePool.hpp` | 0 | 0 | 6 | 6 |
| Foundation | `Foundation/NSBundle.hpp` | 0 | 0 | 8 | 8 |
| Foundation | `Foundation/NSData.hpp` | 0 | 0 | 3 | 3 |
| Foundation | `Foundation/NSDate.hpp` | 0 | 0 | 3 | 3 |
| Foundation | `Foundation/NSDictionary.hpp` | 0 | 0 | 8 | 8 |
| Foundation | `Foundation/NSEnumerator.hpp` | 0 | 0 | 9 | 9 |
| Foundation | `Foundation/NSError.hpp` | 0 | 0 | 23 | 23 |
| Foundation | `Foundation/NSLock.hpp` | 0 | 0 | 10 | 10 |
| Foundation | `Foundation/NSNotification.hpp` | 0 | 0 | 11 | 11 |
| Foundation | `Foundation/NSNumber.hpp` | 0 | 0 | 54 | 54 |
| Foundation | `Foundation/NSObjCRuntime.hpp` | 0 | 0 | 5 | 5 |
| Foundation | `Foundation/NSObject.hpp` | 0 | 0 | 11 | 11 |
| Foundation | `Foundation/NSProcessInfo.hpp` | 0 | 0 | 44 | 44 |
| Foundation | `Foundation/NSRange.hpp` | 0 | 0 | 8 | 8 |
| Foundation | `Foundation/NSSet.hpp` | 0 | 0 | 6 | 6 |
| Foundation | `Foundation/NSSharedPtr.hpp` | 0 | 0 | 13 | 13 |
| Foundation | `Foundation/NSString.hpp` | 0 | 0 | 57 | 57 |
| Foundation | `Foundation/NSTypes.hpp` | 0 | 0 | 10 | 10 |
| Foundation | `Foundation/NSURL.hpp` | 0 | 0 | 4 | 4 |
| Metal | `Metal/MTL4AccelerationStructure.hpp` | 199 | 0 | 0 | 199 |
| Metal | `Metal/MTL4Archive.hpp` | 8 | 0 | 0 | 8 |
| Metal | `Metal/MTL4ArgumentTable.hpp` | 23 | 0 | 0 | 23 |
| Metal | `Metal/MTL4BinaryFunction.hpp` | 3 | 0 | 0 | 3 |
| Metal | `Metal/MTL4BinaryFunctionDescriptor.hpp` | 9 | 0 | 4 | 13 |
| Metal | `Metal/MTL4CommandAllocator.hpp` | 10 | 0 | 0 | 10 |
| Metal | `Metal/MTL4CommandBuffer.hpp` | 22 | 0 | 0 | 22 |
| Metal | `Metal/MTL4CommandEncoder.hpp` | 13 | 0 | 5 | 18 |
| Metal | `Metal/MTL4CommandQueue.hpp` | 29 | 0 | 28 | 57 |
| Metal | `Metal/MTL4CommitFeedback.hpp` | 4 | 0 | 2 | 6 |
| Metal | `Metal/MTL4Compiler.hpp` | 43 | 0 | 10 | 53 |
| Metal | `Metal/MTL4CompilerTask.hpp` | 4 | 0 | 5 | 9 |
| Metal | `Metal/MTL4ComputeCommandEncoder.hpp` | 38 | 0 | 0 | 38 |
| Metal | `Metal/MTL4ComputePipeline.hpp` | 18 | 0 | 0 | 18 |
| Metal | `Metal/MTL4Counters.hpp` | 14 | 0 | 8 | 22 |
| Metal | `Metal/MTL4FunctionDescriptor.hpp` | 3 | 0 | 0 | 3 |
| Metal | `Metal/MTL4LibraryDescriptor.hpp` | 9 | 0 | 0 | 9 |
| Metal | `Metal/MTL4LibraryFunctionDescriptor.hpp` | 7 | 0 | 0 | 7 |
| Metal | `Metal/MTL4LinkingDescriptor.hpp` | 26 | 0 | 0 | 26 |
| Metal | `Metal/MTL4MachineLearningCommandEncoder.hpp` | 4 | 0 | 0 | 4 |
| Metal | `Metal/MTL4MachineLearningPipeline.hpp` | 20 | 0 | 0 | 20 |
| Metal | `Metal/MTL4MeshRenderPipeline.hpp` | 54 | 0 | 0 | 54 |
| Metal | `Metal/MTL4PipelineDataSetSerializer.hpp` | 8 | 0 | 4 | 12 |
| Metal | `Metal/MTL4PipelineState.hpp` | 14 | 0 | 18 | 32 |
| Metal | `Metal/MTL4RenderCommandEncoder.hpp` | 42 | 0 | 5 | 47 |
| Metal | `Metal/MTL4RenderPass.hpp` | 34 | 0 | 0 | 34 |
| Metal | `Metal/MTL4RenderPipeline.hpp` | 78 | 0 | 3 | 81 |
| Metal | `Metal/MTL4SpecializedFunctionDescriptor.hpp` | 9 | 0 | 0 | 9 |
| Metal | `Metal/MTL4StitchedFunctionDescriptor.hpp` | 7 | 0 | 0 | 7 |
| Metal | `Metal/MTL4TileRenderPipeline.hpp` | 19 | 0 | 0 | 19 |
| Metal | `Metal/MTLAccelerationStructure.hpp` | 257 | 0 | 90 | 347 |
| Metal | `Metal/MTLAccelerationStructureCommandEncoder.hpp` | 34 | 0 | 0 | 34 |
| Metal | `Metal/MTLAccelerationStructureTypes.hpp` | 19 | 0 | 16 | 35 |
| Metal | `Metal/MTLAllocation.hpp` | 2 | 0 | 0 | 2 |
| Metal | `Metal/MTLArgument.hpp` | 116 | 0 | 34 | 150 |
| Metal | `Metal/MTLArgumentEncoder.hpp` | 29 | 0 | 0 | 29 |
| Metal | `Metal/MTLBinaryArchive.hpp` | 16 | 0 | 7 | 23 |
| Metal | `Metal/MTLBlitCommandEncoder.hpp` | 28 | 0 | 6 | 34 |
| Metal | `Metal/MTLBlitPass.hpp` | 19 | 0 | 0 | 19 |
| Metal | `Metal/MTLBuffer.hpp` | 12 | 0 | 0 | 12 |
| Metal | `Metal/MTLCaptureManager.hpp` | 25 | 0 | 7 | 32 |
| Metal | `Metal/MTLCaptureScope.hpp` | 5 | 0 | 0 | 5 |
| Metal | `Metal/MTLCommandBuffer.hpp` | 55 | 0 | 35 | 90 |
| Metal | `Metal/MTLCommandEncoder.hpp` | 9 | 0 | 23 | 32 |
| Metal | `Metal/MTLCommandQueue.hpp` | 19 | 0 | 0 | 19 |
| Metal | `Metal/MTLComputeCommandEncoder.hpp` | 40 | 0 | 8 | 48 |
| Metal | `Metal/MTLComputePass.hpp` | 21 | 0 | 0 | 21 |
| Metal | `Metal/MTLComputePipeline.hpp` | 57 | 0 | 0 | 57 |
| Metal | `Metal/MTLCounters.hpp` | 21 | 0 | 43 | 64 |
| Metal | `Metal/MTLDataType.hpp` | 0 | 0 | 98 | 98 |
| Metal | `Metal/MTLDepthStencil.hpp` | 33 | 0 | 18 | 51 |
| Metal | `Metal/MTLDevice.hpp` | 181 | 0 | 122 | 303 |
| Metal | `Metal/MTLDrawable.hpp` | 8 | 0 | 2 | 10 |
| Metal | `Metal/MTLDynamicLibrary.hpp` | 6 | 0 | 7 | 13 |
| Metal | `Metal/MTLEvent.hpp` | 21 | 0 | 2 | 23 |
| Metal | `Metal/MTLFence.hpp` | 4 | 0 | 0 | 4 |
| Metal | `Metal/MTLFunctionConstantValues.hpp` | 7 | 0 | 0 | 7 |
| Metal | `Metal/MTLFunctionDescriptor.hpp` | 17 | 0 | 8 | 25 |
| Metal | `Metal/MTLFunctionHandle.hpp` | 5 | 0 | 0 | 5 |
| Metal | `Metal/MTLFunctionLog.hpp` | 11 | 0 | 2 | 13 |
| Metal | `Metal/MTLFunctionStitching.hpp` | 44 | 0 | 5 | 49 |
| Metal | `Metal/MTLGPUAddress.hpp` | 0 | 0 | 1 | 1 |
| Metal | `Metal/MTLHeap.hpp` | 41 | 0 | 4 | 45 |
| Metal | `Metal/MTLIOCommandBuffer.hpp` | 20 | 0 | 7 | 27 |
| Metal | `Metal/MTLIOCommandQueue.hpp` | 26 | 0 | 11 | 37 |
| Metal | `Metal/MTLIOCompressor.hpp` | 4 | 0 | 4 | 8 |
| Metal | `Metal/MTLIndirectCommandBuffer.hpp` | 47 | 0 | 13 | 60 |
| Metal | `Metal/MTLIndirectCommandEncoder.hpp` | 35 | 0 | 0 | 35 |
| Metal | `Metal/MTLIntersectionFunctionTable.hpp` | 18 | 0 | 17 | 35 |
| Metal | `Metal/MTLLibrary.hpp` | 103 | 0 | 54 | 157 |
| Metal | `Metal/MTLLinkedFunctions.hpp` | 12 | 0 | 0 | 12 |
| Metal | `Metal/MTLLogState.hpp` | 10 | 0 | 12 | 22 |
| Metal | `Metal/MTLParallelRenderCommandEncoder.hpp` | 8 | 0 | 0 | 8 |
| Metal | `Metal/MTLPipeline.hpp` | 10 | 0 | 8 | 18 |
| Metal | `Metal/MTLPixelFormat.hpp` | 0 | 0 | 141 | 141 |
| Metal | `Metal/MTLRasterizationRate.hpp` | 47 | 0 | 0 | 47 |
| Metal | `Metal/MTLRenderCommandEncoder.hpp` | 132 | 0 | 67 | 199 |
| Metal | `Metal/MTLRenderPass.hpp` | 106 | 0 | 31 | 137 |
| Metal | `Metal/MTLRenderPipeline.hpp` | 254 | 0 | 58 | 312 |
| Metal | `Metal/MTLResidencySet.hpp` | 22 | 0 | 0 | 22 |
| Metal | `Metal/MTLResource.hpp` | 15 | 0 | 39 | 54 |
| Metal | `Metal/MTLResourceStateCommandEncoder.hpp` | 7 | 0 | 12 | 19 |
| Metal | `Metal/MTLResourceStatePass.hpp` | 19 | 0 | 0 | 19 |
| Metal | `Metal/MTLResourceViewPool.hpp` | 13 | 0 | 0 | 13 |
| Metal | `Metal/MTLSampler.hpp` | 41 | 0 | 22 | 63 |
| Metal | `Metal/MTLStageInputOutputDescriptor.hpp` | 39 | 0 | 65 | 104 |
| Metal | `Metal/MTLTensor.hpp` | 66 | 0 | 32 | 98 |
| Metal | `Metal/MTLTexture.hpp` | 104 | 0 | 34 | 138 |
| Metal | `Metal/MTLTextureViewPool.hpp` | 4 | 0 | 0 | 4 |
| Metal | `Metal/MTLTypes.hpp` | 16 | 0 | 17 | 33 |
| Metal | `Metal/MTLVersion.hpp` | 0 | 0 | 4 | 4 |
| Metal | `Metal/MTLVertexDescriptor.hpp` | 35 | 0 | 61 | 96 |
| Metal | `Metal/MTLVisibleFunctionTable.hpp` | 10 | 0 | 0 | 10 |
| MetalFX | `MetalFX/MTL4FXFrameInterpolator.hpp` | 0 | 2 | 0 | 2 |
| MetalFX | `MetalFX/MTL4FXSpatialScaler.hpp` | 0 | 2 | 0 | 2 |
| MetalFX | `MetalFX/MTL4FXTemporalDenoisedScaler.hpp` | 0 | 2 | 0 | 2 |
| MetalFX | `MetalFX/MTL4FXTemporalScaler.hpp` | 0 | 2 | 0 | 2 |
| MetalFX | `MetalFX/MTLFXFrameInterpolator.hpp` | 0 | 109 | 0 | 109 |
| MetalFX | `MetalFX/MTLFXSpatialScaler.hpp` | 5 | 39 | 0 | 44 |
| MetalFX | `MetalFX/MTLFXTemporalDenoisedScaler.hpp` | 0 | 136 | 0 | 136 |
| MetalFX | `MetalFX/MTLFXTemporalScaler.hpp` | 5 | 105 | 0 | 110 |
| QuartzCore | `QuartzCore/CAMetalDrawable.hpp` | 2 | 0 | 0 | 2 |
| QuartzCore | `QuartzCore/CAMetalLayer.hpp` | 3 | 0 | 17 | 20 |

## Declaration-kind summary

The machine-readable inventory preserves the original qualified name
and signature for every class, method, enum, option, field, constant,
alias, function, and macro.

| Framework | Kind | Implemented | Availability | Substitute | Total |
| --- | --- | ---: | ---: | ---: | ---: |
| Foundation | `alias` | 0 | 0 | 14 | 14 |
| Foundation | `class` | 0 | 0 | 24 | 24 |
| Foundation | `constant` | 0 | 0 | 28 | 28 |
| Foundation | `enum` | 0 | 0 | 3 | 3 |
| Foundation | `enum_member` | 0 | 0 | 47 | 47 |
| Foundation | `field` | 0 | 0 | 9 | 9 |
| Foundation | `macro` | 0 | 0 | 1 | 1 |
| Foundation | `method` | 0 | 0 | 171 | 171 |
| Foundation | `options` | 0 | 0 | 2 | 2 |
| Foundation | `struct` | 0 | 0 | 3 | 3 |
| Metal | `alias` | 0 | 0 | 68 | 68 |
| Metal | `class` | 232 | 0 | 0 | 232 |
| Metal | `constant` | 0 | 0 | 27 | 27 |
| Metal | `enum` | 0 | 0 | 104 | 104 |
| Metal | `enum_member` | 0 | 0 | 918 | 918 |
| Metal | `field` | 0 | 0 | 152 | 152 |
| Metal | `function` | 9 | 0 | 0 | 9 |
| Metal | `macro` | 0 | 0 | 4 | 4 |
| Metal | `method` | 2915 | 0 | 0 | 2915 |
| Metal | `options` | 0 | 0 | 24 | 24 |
| Metal | `struct` | 0 | 0 | 42 | 42 |
| MetalFX | `class` | 4 | 13 | 0 | 17 |
| MetalFX | `enum` | 0 | 1 | 0 | 1 |
| MetalFX | `enum_member` | 0 | 3 | 0 | 3 |
| MetalFX | `method` | 6 | 380 | 0 | 386 |
| QuartzCore | `class` | 2 | 0 | 0 | 2 |
| QuartzCore | `method` | 3 | 0 | 17 | 20 |

The machine-readable ledger is checked with:

```sh
python3 scripts/generate_api_inventory.py --reference ../D3D_Documentation/metal-cpp --check
python3 scripts/check-api-coverage.py
```

A changed reference digest requires regenerating the inventory and reviewing every new or removed ID.
