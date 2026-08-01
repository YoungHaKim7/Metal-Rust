# Releasing Metal-Rust

Metal-Rust is distributed as two crates with the same version:

- `metal-rust-ffi`: the audited implementation boundary required by Cargo's
  registry resolver. It contains all Objective-C unsafe operations but exports
  safe Rust APIs only.
- `metal-rust`: the application-facing safe API.

The package names are allocated by crates.io only when the first version is
successfully published. Search results are not a reservation, so verify name
availability immediately before the first release.

## Prepare a release

1. Update both workspace package versions and `Cargo.lock`.
2. Add a curated entry to `CHANGELOG.md` once the project begins shipping
   releases.
3. Run the complete source checks:

   ```sh
   cargo fmt --all -- --check
   cargo test --workspace
   cargo clippy --workspace --all-targets -- -D warnings
   scripts/check-commit-message.sh --self-test
   git diff --check
   ```

4. Inspect and verify the implementation package:

   ```sh
   cargo package -p metal-rust-ffi --list
   cargo publish -p metal-rust-ffi --dry-run
   ```

5. Before the matching `metal-rust-ffi` version is present on crates.io, inspect
   the facade package contents:

   ```sh
   cargo package -p metal-rust --list
   ```

   `--no-verify` does not bypass dependency resolution, so the facade cannot be
   packaged until the exact FFI version is visible in the registry. The release
   publication sequence must perform its full dry-run at that point.

6. Commit the release as `chore(release): prepare vX.Y.Z`, create an annotated
   `vX.Y.Z` tag, and push the commit and tag.

## Publish

GitHub release automation is intentionally deferred. When it is added, it must
use the protected `crates-io` environment and the existing
`CARGO_RELEASE_TOKEN` secret. The workflow must validate the tag and package
versions, publish `metal-rust-ffi`, wait until Cargo can resolve the new
version, dry-run `metal-rust`, and only then publish `metal-rust`.

crates.io publication is permanent: a version cannot be overwritten or
deleted. A broken release can only be yanked, so never bypass the dry-runs or
publish from a dirty worktree.
