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
- APIs whose safety depends on GPU synchronization are not exposed until a
  safe synchronization token can express the precondition.

The first slice covers the default `MTLDevice`, device-wide
`MTLCaptureManager` sessions, and checked Metal 4 timestamp CounterHeap creation
and CPU resolution.

## Source layout

Framework directories and API filenames intentionally track metal-cpp:

```text
src/
  Foundation/
    Foundation.rs
    NSError.rs
  Metal/
    Metal.rs
    MTLDevice.rs
    MTLCaptureManager.rs
    MTL4Counters.rs
  MetalFX/
    MetalFX.rs
  QuartzCore/
    QuartzCore.rs
```

The implementation-only `metal-rust-ffi/src/` tree uses the same
Foundation/Metal mapping,
so an API has a predictable public facade and Objective-C implementation file.
Rust module names remain snake_case, while source filenames retain Apple's
framework spelling for direct comparison with metal-cpp and SDK declarations.

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
```

See [RELEASING.md](RELEASING.md) for the crates.io publication order and
release checks. GitHub CI and release automation are intentionally deferred.
