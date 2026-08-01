# Metal-Rust Repository Guide

Metal-Rust is a standalone Cargo workspace. It is not part of the DXMT Git
history even though its contribution policy intentionally follows DXMT.

## Architecture and safety

- `metal-rust` is the public safe facade and must keep
  `#![forbid(unsafe_code)]`.
- `crates/metal-rust-ffi` is the only crate allowed to contain unsafe Rust.
  Its public API must still be safe and must not expose Objective-C pointers,
  selectors, retain/release operations, unchecked buffers, or raw `NSError`
  parameters.
- Every unsafe block needs an adjacent `SAFETY` comment stating the local
  invariant. Keep `unsafe_op_in_unsafe_fn = "deny"` enabled.
- Keep framework directories and Apple API filenames aligned with metal-cpp.
  Rust module names remain snake_case through `#[path]` declarations.

## Commit policy

Use a reviewed Conventional Commit subject for every non-merge commit:

```text
type(scope): concise imperative summary
```

The single source of truth for accepted types and scopes is
`scripts/conventional-commit-keywords.sh`. Add a keyword there before using it.
Do not combine scopes. Prefer one architectural change per commit and keep
formatting-only or generated changes with the change that requires them.

Examples:

- `feat(metal): add safe command queue creation`
- `fix(capture): stop sessions when setup fails`
- `docs(release): document crates.io publication order`
- `refactor(ffi): centralize Objective-C ownership conversion`

Run `scripts/install-git-hooks.sh` to enable the repository-local commit-msg
hook. Add equivalent CI enforcement when the repository CI design is finalized.

## Verification

Run the narrowest relevant test while developing. Before handoff, run:

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
scripts/check-commit-message.sh --self-test
git diff --check
```

Changes to manifests or release metadata must also run the package dry-runs
documented in `RELEASING.md`.

## Publishing

- Publish only from an annotated `vX.Y.Z` tag whose version matches both Cargo
  packages.
- Publish `metal-rust-ffi` before `metal-rust`; the facade cannot be verified by
  crates.io until the exact implementation version is visible in the index.
- Never publish with `--allow-dirty`, and never reuse a published version.
- Treat `CARGO_RELEASE_TOKEN` as a CI secret. Do not write it into repository
  files, shell history, logs, or documentation examples.
