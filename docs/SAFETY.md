# Safety policy

## Boundary

`metal-rust` and its tests forbid unsafe Rust. `metal-rust-ffi` is the only
crate where unsafe is permitted. The FFI crate still exposes safe functions;
unsafe is an implementation detail and not an application requirement.

## Required invariants

Every unsafe block must have a nearby `SAFETY:` comment that states the local
invariant. In particular, wrappers must establish:

- Objective-C object identity and retained/borrowed ownership;
- selector availability before invoking APIs introduced by newer SDKs;
- non-nil results and correct Objective-C class/protocol identity;
- checked length, alignment, range, and integer conversions;
- callback capture lifetime, execution thread, release order, and panic policy;
- GPU/CPU synchronization before exposing mapped resource contents.

No `Send` or `Sync` implementation is added without a framework-backed proof.
No raw Objective-C pointer, selector, `NSError **`, or unchecked GPU address is
part of the public facade.

## Audit checks

The repository check suite in `scripts/check-safety.py` scans the facade and
tests for unsafe syntax, checks `SAFETY:` coverage for FFI unsafe blocks, and
rejects raw `objc2` mentions in public source. Runtime availability failures
are represented as owned errors/capabilities rather than selector crashes.

The crate-level rustdoc contains a `compile_fail` visibility fixture that tries
to extract the private implementation field from `Device`. This is run by
`cargo test` and guards the public ownership boundary in addition to the
source audit.
