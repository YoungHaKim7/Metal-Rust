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
explicitly committed and can then be synchronously waited on. The wait returns
a lifetime-bound `CompletionToken`; GPU-written buffer, texture, and counter
readback requires that token, so the synchronization precondition is visible
in the Rust type signature.

Resource helpers never expose a raw mapped pointer. Buffer writes require a
CPU-visible storage mode and a checked range. Texture readback requires a
checked region and row stride, and callers receive an owned byte vector. The
facade does not add `Send` or `Sync` implementations for Metal objects.

The implementation is organized in vertical slices. A slice is complete only
when its FFI wrapper, safe facade, availability/error behavior, documentation,
inventory mapping, and tests land together.
