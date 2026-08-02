# Metal-Rust

Safe Rust interfaces for Apple Metal. The API follows the same framework and
protocol surface as Apple's metal-cpp, while making Objective-C ownership and
message-send invariants private implementation details.

## Safety boundary

- The public `metal-rust` crate uses `#![forbid(unsafe_code)]`.
- Raw Objective-C interop is confined to the separately audited
  `metal-rust-ffi` workspace crate. That implementation crate is published only
  so package registries can resolve `metal-rust`; applications should depend on
  `metal-rust`, and both crates expose safe Rust APIs only.
- Public APIs return owned Rust values and typed RAII objects. They never expose
  Objective-C pointers, selectors, `NSError **`, retain/release, or unchecked
  ranges.
- APIs whose safety depends on GPU synchronization remain unavailable until
  resource use and command completion can be tied together by the type system.

The safe facade covers the core Metal submission/resource path, Metal 4
timestamp CounterHeap ownership, device-wide capture, MetalFX spatial and
temporal scaler descriptors, and CAMetalLayer drawable ownership. The full
metal-cpp reference inventory is tracked in
[`api/metal-cpp-inventory.json`](api/metal-cpp-inventory.json), with one
explicit implemented or unimplemented status per declaration in
[`api/coverage.json`](api/coverage.json). Only audited facade mappings count as
coverage; a transitive generated binding does not.

The audited generated layer currently supplies owned opaque wrappers, safe
scalar and struct representations, non-callback aliases, runtime-gated default
constructors, and checked property accessors. Generated `try_*` getters return
`None` when a selector is unavailable; enum setters reject undeclared values,
floating-point setters reject non-finite values, and object/string setters keep
all Objective-C ownership inside `metal-rust-ffi`. Methods involving callbacks,
raw buffers, synchronization, or API-specific numeric constraints remain
explicitly `unimplemented` until their contracts are represented safely.

## Source layout

Framework directories and API filenames intentionally track metal-cpp:

```text
src/
  Foundation/
    Foundation.rs
    NSError.rs
  Metal/
    Metal.rs
    MTLBuffer.rs
    MTLBlitCommandEncoder.rs
    MTLCommandBuffer.rs
    MTLCommandEncoder.rs
    MTLCommandQueue.rs
    MTLRenderPass.rs
    MTLShader.rs
    MTLTexture.rs
    MTLTypes.rs
    MTLDevice.rs
    MTLCaptureManager.rs
    MTL4Counters.rs
  MetalFX/
    MetalFX.rs
    MTLFXSpatialScaler.rs
    MTLFXTemporalScaler.rs
  QuartzCore/
    CAMetalLayer.rs
    QuartzCore.rs
```

The implementation-only `metal-rust-ffi/src/` tree uses the same
Foundation/Metal mapping,
so an API has a predictable public facade and Objective-C implementation file.
Rust module names remain snake_case, while source filenames retain Apple's
framework spelling for direct comparison with metal-cpp and SDK declarations.

## Safe workflow

The normal workflow is entirely safe Rust: create a `Device` and
`CommandQueue`, allocate shared `Buffer`/`Texture` resources, compile a
`Library` and `Function`, build a pipeline, encode render/compute/blit work,
then commit and wait on the `CommandBuffer`. GPU-written resource readback is
intentionally not exposed yet: completion of an unrelated command buffer is
not proof that a particular resource is idle. `Layer::next_drawable` and
`CommandBuffer::present_drawable` cover presentation when Core Animation can
provide a drawable. `CaptureSession`, MetalFX descriptors, and the Metal 4
counter heap return owned `Error` values when the installed runtime does not
support the requested capability. No resource readback API exposes mapped GPU
memory before a resource-specific synchronization model is implemented.

## Reference

The API organization and naming are informed by Apple's metal-cpp distribution.
metal-cpp and this project are licensed under Apache License 2.0. Objective-C
binding generation is provided internally by the Apache/MIT/Zlib-licensed
`objc2` project; its raw bindings are not re-exported.

## Development

Use Conventional Commit subjects in the form `type(scope): summary`. The
reviewed types and scopes live in
[`scripts/conventional-commit-keywords.sh`](scripts/conventional-commit-keywords.sh),
and the repository hook validates local commit subjects. Install it with:

```sh
scripts/install-git-hooks.sh
```

Before submitting a change, run:

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
scripts/check-commit-message.sh --self-test
python3 scripts/check-api-coverage.py
python3 scripts/check-safety.py
```

See [RELEASING.md](RELEASING.md) for the crates.io publication order and
release checks. General pull-request CI is intentionally deferred; releases
use a manual workflow on the organization's macOS runner.
