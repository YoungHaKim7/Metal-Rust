#!/usr/bin/env python3
"""Validate the explicit metal-cpp-to-safe-Rust coverage mapping."""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any


VALID_STATUSES = {"implemented", "availability", "substitute"}


def load(path: Path) -> dict[str, Any]:
    with path.open(encoding="utf-8") as handle:
        value = json.load(handle)
    if not isinstance(value, dict):
        raise ValueError(f"{path} must contain a JSON object")
    return value


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--inventory", type=Path, default=Path("api/metal-cpp-inventory.json"))
    parser.add_argument("--coverage", type=Path, default=Path("api/coverage.json"))
    args = parser.parse_args()

    inventory = load(args.inventory)
    coverage = load(args.coverage)
    declarations = inventory.get("declarations")
    mappings = coverage.get("mappings")
    if not isinstance(declarations, list) or not isinstance(mappings, dict):
        print("inventory.declarations and coverage.mappings are required", file=sys.stderr)
        return 2

    if coverage.get("reference_sha256") != inventory.get("reference", {}).get("content_sha256"):
        print("coverage reference_sha256 does not match inventory reference digest", file=sys.stderr)
        return 1

    declaration_by_id: dict[str, dict[str, Any]] = {}
    errors: list[str] = []
    for declaration in declarations:
        identifier = declaration.get("id")
        if not isinstance(identifier, str) or not identifier:
            errors.append("inventory contains a declaration without an id")
            continue
        if identifier in declaration_by_id:
            errors.append(f"duplicate inventory id: {identifier}")
        declaration_by_id[identifier] = declaration

    unknown = sorted(set(mappings) - set(declaration_by_id))
    errors.extend(f"coverage mapping has unknown inventory id: {identifier}" for identifier in unknown)

    by_framework: dict[str, Counter[str]] = defaultdict(Counter)
    by_kind: Counter[str] = Counter()
    for identifier, declaration in declaration_by_id.items():
        mapping = mappings.get(identifier)
        if not isinstance(mapping, dict):
            errors.append(f"missing coverage mapping: {identifier}")
            continue
        status = mapping.get("status")
        rust = mapping.get("rust")
        if status not in VALID_STATUSES:
            errors.append(
                f"{identifier}: status must be one of {sorted(VALID_STATUSES)}, got {status!r}"
            )
        if not isinstance(rust, str) or not rust.strip():
            errors.append(f"{identifier}: rust mapping is required")
        if not isinstance(mapping.get("evidence"), str) or not mapping["evidence"].strip():
            errors.append(f"{identifier}: evidence is required")
        if status == "substitute" and not isinstance(mapping.get("notes"), str):
            errors.append(f"{identifier}: substitute mappings require notes")
        if status == "availability" and not isinstance(mapping.get("availability"), str):
            errors.append(f"{identifier}: availability mappings require availability")
        by_framework[declaration["framework"]][status or "invalid"] += 1
        by_kind[declaration["kind"]] += 1

    covered = sum(
        count
        for counts in by_framework.values()
        for status, count in counts.items()
        if status in VALID_STATUSES
    )
    total = len(declaration_by_id)
    print(f"inventory declarations: {total}")
    print(f"explicit safe mappings: {covered}")
    print(f"coverage: {covered}/{total} ({(covered / total * 100 if total else 100):.2f}%)")
    for framework in sorted(by_framework):
        print(f"{framework}: {dict(sorted(by_framework[framework].items()))}")

    if errors:
        print(f"coverage check failed with {len(errors)} error(s):", file=sys.stderr)
        for error in errors[:80]:
            print(f"- {error}", file=sys.stderr)
        if len(errors) > 80:
            print(f"- ... {len(errors) - 80} more", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
