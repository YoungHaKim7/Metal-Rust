# Changelog

## 1.0.0 - 2026-08-02

- Expose canonical safe APIs for all 5,226 declarations in the pinned
  metal-cpp inventory across Foundation, Metal, Metal 4, MetalFX, and
  QuartzCore.
- Confine Objective-C messaging, ownership conversion, raw pointers,
  selectors, callback ABI, and GPU synchronization to `metal-rust-ffi`.
- Add checked command-buffer typestates, RAII callback registrations, and
  submission-bound staging readback for buffers, textures, and counters.
- Add availability-aware MetalFX and Metal 4 APIs plus safe Rust-native
  Foundation values and collections.
- Add inventory, coverage, generator-freshness, safety-boundary, Send/Sync,
  compile-fail, package, and release validation.
