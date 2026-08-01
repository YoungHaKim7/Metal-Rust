#!/usr/bin/env python3
"""Run narrow source audits for the public safety boundary."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


UNSAFE_WORD = re.compile(r"\bunsafe\b")
UNSAFE_BLOCK = re.compile(r"\bunsafe\s*\{")


def rust_files(root: Path) -> list[Path]:
    return sorted(path for path in root.rglob("*.rs") if path.is_file())


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path("."))
    args = parser.parse_args()
    root = args.root
    errors: list[str] = []

    public_files = rust_files(root / "src") + rust_files(root / "tests")
    for path in public_files:
        for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if "#![forbid(unsafe_code)]" in line:
                continue
            code = line.split("//", 1)[0]
            if UNSAFE_WORD.search(code):
                errors.append(f"public safety boundary contains unsafe syntax: {path}:{line_number}")

    ffi_root = root / "crates" / "metal-rust-ffi" / "src"
    for path in rust_files(ffi_root):
        lines = path.read_text(encoding="utf-8").splitlines()
        for index, line in enumerate(lines):
            if not UNSAFE_BLOCK.search(line):
                continue
            context = "\n".join(lines[max(0, index - 8) : index + 1])
            if "SAFETY:" not in context:
                errors.append(f"unsafe block lacks a nearby SAFETY comment: {path}:{index + 1}")

    for path in public_files:
        for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if "objc2" in line:
                errors.append(f"public facade mentions raw objc2 binding: {path}:{line_number}")

    if errors:
        print(f"safety check failed with {len(errors)} error(s)", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print(f"safety check passed: {len(public_files)} public Rust files audited")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
