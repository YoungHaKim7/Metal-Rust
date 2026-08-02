#!/usr/bin/env python3
"""Generate safe scalar Metal enum and option-set wrappers from the inventory."""

from __future__ import annotations

import argparse
import ast
import json
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any


MANUAL_TYPES = {
    "MTL::CommandBufferStatus",
    "MTL::PixelFormat",
    "MTL::PrimitiveType",
    "MTL::ResourceOptions",
    "MTL::StorageMode",
    "MTL::TextureType",
    "MTL::TextureUsage",
}

EXTENSIBLE_MANUAL_TYPES = {
    "MTL::PixelFormat": ("PixelFormat", "u64"),
    "MTL::ResourceOptions": ("ResourceOptions", "u64"),
    "MTL::TextureUsage": ("TextureUsage", "u64"),
}

RUST_INTEGERS = {
    "Integer": "isize",
    "NS::Integer": "isize",
    "UInteger": "usize",
    "NS::UInteger": "usize",
    "uint32_t": "u32",
    "uint8_t": "u8",
    "std::uint64_t": "u64",
}


def load_inventory(path: Path) -> list[dict[str, Any]]:
    value = json.loads(path.read_text(encoding="utf-8"))
    declarations = value.get("declarations")
    if not isinstance(declarations, list):
        raise ValueError("inventory.declarations must be a list")
    return declarations


def generated_type_names(declarations: list[dict[str, Any]]) -> set[str]:
    return {
        declaration["qualified_name"]
        for declaration in declarations
        if declaration["framework"] in {"Foundation", "Metal"}
        and declaration["kind"] in {"enum", "options"}
        and declaration["qualified_name"] not in MANUAL_TYPES
    }


def evaluate_integer(expression: str, known: dict[str, int]) -> int:
    node = ast.parse(expression, mode="eval").body

    def evaluate(item: ast.expr) -> int:
        if isinstance(item, ast.Constant) and isinstance(item.value, int):
            return item.value
        if isinstance(item, ast.Name) and item.id in known:
            return known[item.id]
        if isinstance(item, ast.UnaryOp):
            value = evaluate(item.operand)
            if isinstance(item.op, ast.USub):
                return -value
            if isinstance(item.op, ast.Invert):
                return ~value
        if isinstance(item, ast.BinOp):
            left, right = evaluate(item.left), evaluate(item.right)
            if isinstance(item.op, ast.LShift):
                return left << right
            if isinstance(item.op, ast.BitOr):
                return left | right
            if isinstance(item.op, ast.BitAnd):
                return left & right
        raise ValueError(f"unsupported integer expression: {expression}")

    return evaluate(node)


def parse_integer(
    value: str | None, implicit: int, known: dict[str, int]
) -> tuple[str, int]:
    if value is None:
        return str(implicit), implicit
    if value == "NS::UIntegerMax":
        return "usize::MAX", (1 << 64) - 1
    normalized = re.sub(r"(?<=[0-9A-Fa-f])(ULL|UL|LL|L)\b", "", value)
    number = evaluate_integer(normalized, known)
    return str(number), number


def resolve_members(items: list[dict[str, Any]]) -> dict[str, tuple[str, int]]:
    known: dict[str, int] = {}
    resolved: dict[str, tuple[str, int]] = {}
    pending: list[dict[str, Any]] = []
    implicit = 0
    for item in items:
        try:
            expression, numeric = parse_integer(item["value"], implicit, known)
        except ValueError:
            pending.append(item)
            continue
        resolved[item["name"]] = (expression, numeric)
        known[item["name"]] = numeric
        implicit = numeric + 1
    while pending:
        remaining = []
        for item in pending:
            try:
                expression, numeric = parse_integer(item["value"], implicit, known)
            except ValueError:
                remaining.append(item)
                continue
            resolved[item["name"]] = (expression, numeric)
            known[item["name"]] = numeric
        if len(remaining) == len(pending):
            names = ", ".join(item["qualified_name"] for item in remaining)
            raise ValueError(f"unresolved enum values: {names}")
        pending = remaining
    return resolved


def rust_patterns(values: list[int]) -> str:
    ordered = sorted(set(values))
    patterns: list[str] = []
    start = previous = ordered[0]
    for value in ordered[1:] + [ordered[-1] + 2]:
        if value == previous + 1:
            previous = value
            continue
        if start == previous:
            patterns.append(str(start))
        elif previous == start + 1:
            patterns.extend((str(start), str(previous)))
        else:
            patterns.append(f"{start}..={previous}")
        start = previous = value
    return " | ".join(patterns)


def render(declarations: list[dict[str, Any]]) -> str:
    type_declarations = {
        declaration["qualified_name"]: declaration
        for declaration in declarations
        if declaration["framework"] in {"Foundation", "Metal"}
        and declaration["kind"] in {"enum", "options"}
        and declaration["qualified_name"] not in MANUAL_TYPES
    }
    option_types = {
        declaration["qualified_name"]
        for declaration in declarations
        if declaration["framework"] in {"Foundation", "Metal"}
        and declaration["kind"] == "options"
        and declaration["qualified_name"] not in MANUAL_TYPES
    }
    members: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for declaration in declarations:
        if declaration["kind"] == "enum_member" and declaration.get("parent") in type_declarations:
            members[declaration["parent"]].append(declaration)

    lines = [
        "//! Generated safe Metal scalar enums and option sets.",
        "//!",
        "//! Regenerate with `python3 scripts/generate_value_types.py`.",
        "",
        "#![allow(clippy::upper_case_acronyms)]",
        "#![allow(dead_code)]",
        "#![allow(non_upper_case_globals)]",
        "",
    ]
    for qualified_name, (name, _raw) in EXTENSIBLE_MANUAL_TYPES.items():
        manual_members = sorted(
            (
                declaration
                for declaration in declarations
                if declaration["kind"] == "enum_member"
                and declaration.get("parent") == qualified_name
            ),
            key=lambda item: item["ordinal"],
        )
        resolved_members = resolve_members(manual_members)
        lines += [f"impl super::{name} {{"]
        for member in manual_members:
            expression, _numeric = resolved_members[member["name"]]
            lines += [
                f"    /// Metal value `{member['qualified_name']}`.",
                f"    pub const {member['name']}: Self = Self({expression});",
            ]
        lines += ["}", ""]

    for qualified_name, declaration in sorted(type_declarations.items()):
        name = qualified_name.rsplit("::", 1)[-1]
        raw = RUST_INTEGERS.get(declaration["value"])
        if raw is None:
            raise ValueError(f"unsupported integer type for {qualified_name}: {declaration['value']}")
        kind = "option set" if qualified_name in option_types else "enumeration"
        lines += [
            f"/// Safe scalar wrapper for the Metal `{qualified_name}` {kind}.",
            "#[repr(transparent)]",
            "#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]",
            f"pub struct {name}({raw});",
            "",
            f"impl {name} {{",
        ]
        sorted_members = sorted(members[qualified_name], key=lambda item: item["ordinal"])
        resolved_members = resolve_members(sorted_members)
        member_expressions: list[str] = []
        member_values: list[int] = []
        for member in sorted_members:
            expression, numeric = resolved_members[member["name"]]
            if expression not in member_expressions:
                member_expressions.append(expression)
            if numeric not in member_values:
                member_values.append(numeric)
            lines += [
                f"    /// Metal value `{member['qualified_name']}`.",
                f"    pub const {member['name']}: Self = Self({expression});",
            ]
        lines += [
            "",
            "    /// Preserves a raw value returned by the framework.",
            f"    pub(crate) const fn from_system_raw(value: {raw}) -> Self {{",
            "        Self(value)",
            "    }",
            "",
            "    /// Returns the framework's integer representation.",
            "    #[must_use]",
            f"    pub const fn as_raw(self) -> {raw} {{",
            "        self.0",
            "    }",
            "",
            "    /// Returns whether this is a declared value or valid option-bit combination.",
            "    #[must_use]",
        ]
        if qualified_name in option_types:
            valid_bits = 0
            for member_value in member_values:
                valid_bits |= member_value
            lines += [
                "    pub const fn is_valid(self) -> bool {",
                f"        const VALID_BITS: {raw} = {valid_bits};",
                "        self.0 & !VALID_BITS == 0",
                "    }",
                "",
                "    /// Creates an option set when every bit is declared.",
                "    #[must_use]",
                f"    pub const fn from_bits(value: {raw}) -> Option<Self> {{",
                "        let value = Self(value);",
                "        if value.is_valid() { Some(value) } else { None }",
                "    }",
            ]
        else:
            valid_values = rust_patterns(member_values) if member_values else ""
            lines += [
                "    pub const fn is_valid(self) -> bool {",
                (
                    f"        matches!(self.0, {valid_values})"
                    if valid_values
                    else "        false"
                ),
                "    }",
            ]
        lines += [
            "}",
            "",
        ]
        if qualified_name not in option_types:
            lines += [
                f"impl std::convert::TryFrom<{raw}> for {name} {{",
                "    type Error = ();",
                "",
                f"    fn try_from(value: {raw}) -> Result<Self, Self::Error> {{",
                "        let value = Self(value);",
                "        if value.is_valid() { Ok(value) } else { Err(()) }",
                "    }",
                "}",
                "",
            ]
        if qualified_name in option_types:
            lines += [
                f"impl std::ops::BitOr for {name} {{",
                "    type Output = Self;",
                "",
                "    fn bitor(self, other: Self) -> Self {",
                "        Self(self.0 | other.0)",
                "    }",
                "}",
                "",
                f"impl std::ops::BitAnd for {name} {{",
                "    type Output = Self;",
                "",
                "    fn bitand(self, other: Self) -> Self {",
                "        Self(self.0 & other.0)",
                "    }",
                "}",
                "",
            ]
    source = "\n".join(lines)
    formatted = subprocess.run(
        ["rustfmt", "--edition", "2024", "--emit", "stdout"],
        input=source,
        check=False,
        capture_output=True,
        text=True,
    )
    if formatted.returncode != 0:
        raise RuntimeError(f"rustfmt failed for generated value types: {formatted.stderr}")
    return formatted.stdout


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--inventory", type=Path, default=Path("api/metal-cpp-inventory.json"))
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("crates/metal-rust-ffi/src/Metal/MTLGeneratedValueTypes.rs"),
    )
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()

    declarations = load_inventory(args.inventory)
    output = render(declarations)
    if args.check:
        if not args.output.is_file() or args.output.read_text(encoding="utf-8") != output:
            print(f"generated value types are stale: {args.output}", file=sys.stderr)
            return 1
        print(f"generated value types are current: {len(generated_type_names(declarations))} types")
        return 0
    args.output.write_text(output, encoding="utf-8")
    print(f"generated {len(generated_type_names(declarations))} safe value types")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
