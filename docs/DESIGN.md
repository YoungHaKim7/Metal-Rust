# Safe Rust design

Metal-Rust keeps the framework-oriented source layout of metal-cpp while
changing its ownership model:

| metal-cpp concept | safe Rust mapping |
| --- | --- |
| retained Objective-C object | an owned RAII wrapper in `metal-rust` |
| borrowed object parameter | a shared Rust reference to an owned wrapper |
| `NSError **` | `Result<T, metal_rust::Error>` with owned diagnostics |
| `NS::Range` / `NSUInteger` | checked `Range<usize>` or a typed value with overflow validation |
| option-set typedef | a Rust newtype with explicit constructors and bit operations |
| block callback | a `'static`/thread-checked closure adapter with documented drop behavior |
| raw mapped GPU memory | a lifetime-bound guard or synchronization token, never a naked pointer |
| optional/new selector | a capability query followed by a deterministic safe result |

The `metal-rust-ffi` crate owns Objective-C messages, retained references,
framework conversions, and all unsafe blocks. Its public functions are safe
but deliberately narrow: raw `objc2` objects, selectors, `NSError **`, and
unchecked addresses cannot cross that boundary. The application-facing
`metal-rust` crate has `#![forbid(unsafe_code)]` and exposes only the checked
facade.

Inventory-driven generators may emit safe value types and owned object wrappers,
but generation alone is not method coverage. A generated property method counts
only when its selector is checked at runtime and its ABI shape is restricted to
an audited primitive, validated enum/option set, owned string, or typed RAII
object. Numeric setters without a framework-independent validity rule, callback
methods, and pointer/buffer methods remain unimplemented for handwritten slices.

Overloads that differ only in C++ ownership or callback spelling are combined
when Rust can express the same operation more clearly. Such a replacement is
recorded as `substitute` in `api/coverage.json`; it is not an omission.

## Availability and errors

The facade does not assume that an Apple framework or GPU capability exists.
Optional MetalFX creation is exposed as `Result` and returns
`ErrorKind::Unsupported` when the runtime capability query or factory returns
no object. The same rule applies to drawable acquisition and Metal 4 features.
Argument checks happen before entering the FFI boundary, including dimensions,
mip counts, resource ranges, binding indices, and dispatch sizes. Objective-C
`NSError` diagnostics are copied into owned Rust strings and returned through
`Error` rather than leaked as pointers.

## Lifecycle and synchronization

Owned wrappers retain their underlying framework objects and release them by
normal Rust drop order. A command encoder borrows its command buffer mutably;
the borrow prevents commit or a second encoder while encoding is active. Drop
also ends an encoder that was not explicitly ended. A command buffer is
explicitly committed and can then be synchronously waited on. Resource
readback remains unavailable until the implementation can prove that the
specific resource, rather than merely an arbitrary command buffer, has
completed all GPU access.

Resource helpers never expose a raw mapped pointer. Buffer writes require a
CPU-visible storage mode and a checked range. Buffer, texture, and counter
readback will be added only with resource-specific synchronization and complete
layout validation.

## Thread-safety policy

Every owned Objective-C object wrapper is `!Send + !Sync` by default. The FFI
wrapper carries a private `ThreadBound` marker, and the facade inherits that
property by owning the FFI wrapper. This is deliberate even when the current
binding type would otherwise acquire an automatic `Send` or `Sync`
implementation.

The explicit cross-thread allowlist is currently empty. Adding an entry
requires a per-type audit that cites Apple's thread-safety contract, checks all
methods exposed by Metal-Rust, and places an English `SAFETY` explanation next
to the internal implementation. General Objective-C retain/release thread
safety, callback delivery on arbitrary queues, or observed application behavior
is not sufficient evidence. Public unsafe traits or caller-provided thread
safety promises are never part of this mechanism.

The implementation is organized in vertical slices. A slice is complete only
when its FFI wrapper, safe facade, availability/error behavior, documentation,
inventory mapping, and tests land together.
