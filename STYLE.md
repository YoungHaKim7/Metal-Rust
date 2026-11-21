# Rust House Style

Applies to every Rust file written or edited. When a file is sent for
reorganization, reorder it into the shape below, keep the four-group `use`
block, and apply the spacing rules.

---

## 1. File header (before the `use` block)

In this exact order:

1. `//!` module-level doc comments — very first, nothing above them
2. `#![...]` inner attributes (`#![allow(...)]`, `#![cfg_attr(...)]`, etc.)
3. `extern crate` (rare in Rust 2021+, but if present it goes here, above `use`)

```rust
//! Module summary.
//!
//! Longer description.

#![allow(clippy::all)]

extern crate foo;

use std::…;
```

---

## 2. Top-of-file `use` block

Four source groups, in this order, separated by a blank line:

1. `std` — standard library
2. `external` — non-windows third-party crates
3. `windows` — the `windows` crate
4. `crate` — own crate

```rust
use std::{ … };

use anyhow::…;
use log::…;
use serde::{ … };

use windows::{ … };

use crate::{ … };
```

`pub use` follows the same source grouping — `pub use std::…` sits in the
std group, `pub use crate::…` in the crate group, etc.

### 2.1 Brace grouping — within a single root

**Within one crate root, collapse all imports into a single `use` statement
using braces.** The goal is one `use` statement per crate root, not one
line per import.

- **Same root prefix → `root::{ … }` form.** All imports sharing a common
  prefix collapse into one statement.

  ```rust
  // Ungrouped — one line per import
  use std::collections::HashMap;
  use std::io::Read;
  use std::io::Write;
  use std::path::PathBuf;

  // Preferred — single std statement
  use std::{
      collections::HashMap,
      io::{Read, Write},
      path::PathBuf,
  };
  ```

- **Different crate roots → keep separate statements.** Each external crate
  gets its own `use` line, so they don't get visually mixed. Brace-grouping
  still applies *within* a single crate root:

  ```rust
  // Preferred — separate statements, one per external crate root,
  // sorted alphabetically by crate name
  use anyhow::Error;
  use log::info;
  use serde::{Deserialize, Serialize};
  ```

  Do **not** collapse external crates into `use { … };` — that form mixes
  different roots and obscures which crate each import comes from.

- Apply the same rule to `pub use`. Within a source group, `use` comes
  before `pub use`:

  ```rust
  // std group
  use std::{
      collections::HashMap,
      io::{Read, Write},
  };
  pub use std::{
      fmt,
      path::PathBuf,
  };
  ```

### 2.2 Within a brace group, ordering

1. By path, alphabetically (segment by segment).
2. When a segment has both a leaf item and sub-paths, merge them with
   `self` — matches `rustfmt` defaults.

  ```rust
  use std::{
      collections::HashMap,
      io::{self, Read, Write},   // `io` itself + items under `io::`
      path::PathBuf,
  };
  ```

### 2.3 Source-group order, recap

```
std  →  external  →  windows  →  crate
```

Blank line between groups; no blank line between statements inside the same
group.

---

## 3. Module-level item order

| # | Category | Notes |
|---|----------|-------|
| 1 | `//!` module docs | file header, nothing above |
| 2 | `#![...]` inner attributes | |
| 3 | `extern crate` | rare in 2021+ |
| 4 | `use` / `pub use` | std → external → windows → crate |
| 5 | `mod` / `pub mod` | declarations |
| 6 | `extern "…" { }` blocks | FFI item declarations |
| 7 | `const` / `static` | no blank line between them — see §4 |
| 8 | `type` aliases | |
| 9 | `macro_rules!` | if any |
| 10 | `trait` | |
| 11 | `struct` / `enum` / `union` | same category |
| 12 | `impl` | |
| 13 | `fn` | safe fns: `fn`, `pub fn`, `async fn`, `const fn`, safe `extern "…" fn` |
| 14 | `unsafe fn` | plain `unsafe fn` (non-FFI) |
| 15 | `unsafe extern "…" fn` | FFI / callbacks (e.g. `WNDPROC`) |
| 16 | `fn main` | binary entry files only (`main.rs`, `src/bin/*.rs`) |
| 17 | `#[cfg(test)] mod tests` | conventionally last, when kept in same file |

---

## 4. Spacing rules

- **One blank line** between top-level items of **different categories**.
- **One blank line** between adjacent items of the **same category**
  (e.g. two `struct`s, two `fn`s).
- **Exception:** `const` and `static` are placed **without** a blank line
  between them — they form a single "constants" block.
- **No blank line** within a single item (between its attributes, signature,
  and body).

---

## 5. Within-category ordering

### 5.1 Visibility (within a single category)

Order from widest to narrowest:

```
pub  →  pub(crate)  →  pub(super)  →  pub(self) / private
```

### 5.2 Attributes (on a single item)

Order outer attributes top-to-bottom:

1. `#[cfg(...)]` / `#[cfg_attr(...)]`
2. `#[allow(...)]` / `#[warn(...)]` / `#[deny(...)]` / `#[forbid(...)]`
3. `#[derive(...)]`
4. other attributes (`#[inline]`, `#[repr(...)]`, doc comments `///`, etc.)

```rust
#[cfg(windows)]
#[allow(dead_code)]
#[derive(Debug, Clone)]
#[repr(C)]
/// A window handle.
pub struct HWND(pub *mut core::ffi::c_void);
```

---

## 6. Canonical skeleton

```rust
//! Module summary.

#![allow(...)]

extern crate foo;

use std::{
    collections::HashMap,
    io::{self, Read, Write},
};

use anyhow::Error;
use log::info;
use serde::{Deserialize, Serialize};

use windows::{
    Win32::Foundation::{HANDLE, HWND},
    Win32::UI::WindowsAndMessaging::*,
};

use crate::{
    types::MyType,
    utils::helper,
};
pub use crate::types::PublicType;

mod foo;

extern "C" {
    fn ext_fn(x: i32) -> i32;
}

const A: i32 = 1;
static B: i32 = 2;

type MyInt = i32;

macro_rules! foo { … }

trait Bar { … }

enum Baz { … }
struct Quux { … }
union U { … }

impl Quux { … }

fn safe_fn() { … }

unsafe fn plain_unsafe() { … }

unsafe extern "system" fn callback(_: *mut core::ffi::c_void) -> i32 { … }

fn main() { … }

#[cfg(test)]
mod tests { … }
```

---

## 7. Reorganization procedure

When reordering an existing file **in place** (no file moves):

1. Collect every top-level item, preserving its body verbatim.
2. Bucket each item by category from the table in §3.
3. Within each bucket, sort by §5 (visibility, then name).
4. Reattach attributes per §5.2.
5. Emit buckets in order, applying §4 spacing.
6. Rebuild the `use` block per §2:
   - Four source groups (std → external → windows → crate), blank line
     between groups.
   - Within each group, brace-group all imports sharing a single crate
     root into one `use root::{ … };` statement (§2.1).
   - Different crate roots stay as **separate** `use` statements — do
     **not** collapse them into `use { … };`. Each external crate gets
     its own line, sorted alphabetically by crate name.
   - Within each group, `use` before `pub use`.
   - Order paths inside braces per §2.2.
7. Do not rename, merge, or rewrite any non-`use` item — only reorder and
   re-space. (`use` statements are the sole exception: they are merged and
   reformatted per §2.)
