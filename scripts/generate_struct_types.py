#!/usr/bin/env python3
"""Generate safe Rust representations for plain metal-cpp structs."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any


MANUAL_STRUCTS = {
    "NS::FastEnumerationState",
    "MTL::ClearColor",
    "MTL::Origin",
    "MTL::Region",
    "MTL::Size",
    "MTL::Viewport",
}

SPECIAL_STRUCTS = {
    "MTL::PackedFloat3": ("f32", 3),
    "MTL::PackedFloatQuaternion": ("f32", 4),
}

PACKED_COMPONENTS = {
    3: ("x", "y", "z"),
    4: ("x", "y", "z", "w"),
}

SCALARS = {
    "double": "f64",
    "float": "f32",
    "int32_t": "i32",
    "NS::Integer": "isize",
    "NS::UInteger": "usize",
    "Integer": "isize",
    "UInteger": "usize",
    "uint16_t": "u16",
    "uint32_t": "u32",
    "uint64_t": "u64",
}


def load_inventory(path: Path) -> list[dict[str, Any]]:
    value = json.loads(path.read_text(encoding="utf-8"))
    declarations = value.get("declarations")
    if not isinstance(declarations, list):
        raise ValueError("inventory.declarations must be a list")
    return declarations


def generated_struct_names(declarations: list[dict[str, Any]]) -> set[str]:
    return {
        declaration["qualified_name"]
        for declaration in declarations
        if declaration["framework"] in {"Foundation", "Metal"}
        and declaration["kind"] == "struct"
        and declaration["qualified_name"] not in MANUAL_STRUCTS
    }


def snake_case(name: str) -> str:
    value = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", name)
    return value.lower()


def rust_type(cpp_type: str) -> str:
    array = re.fullmatch(r"(.+?)\[(\d+)\]", cpp_type)
    if array:
        return f"[{rust_type(array.group(1).strip())}; {array.group(2)}]"
    if cpp_type == "NS::Range":
        return "std::ops::Range<usize>"
    if cpp_type in SCALARS:
        return SCALARS[cpp_type]
    if cpp_type.startswith("MTL::") or cpp_type.startswith("MTL4::"):
        return cpp_type.rsplit("::", 1)[-1]
    if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", cpp_type):
        return cpp_type
    raise ValueError(f"unsupported struct field type: {cpp_type}")


def parse_field(signature: str) -> tuple[str, str]:
    match = re.fullmatch(r"(.+?)\s+([A-Za-z_][A-Za-z0-9_]*)(\[\d+\])?;", signature)
    if not match:
        raise ValueError(f"unsupported struct field: {signature}")
    cpp_type = match.group(1).strip() + (match.group(3) or "")
    return snake_case(match.group(2)), rust_type(cpp_type)


def render(declarations: list[dict[str, Any]]) -> str:
    structs = {
        declaration["qualified_name"]: declaration
        for declaration in declarations
        if declaration["framework"] in {"Foundation", "Metal"}
        and declaration["kind"] == "struct"
        and declaration["qualified_name"] not in MANUAL_STRUCTS
    }
    fields: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for declaration in declarations:
        if declaration["kind"] == "field" and declaration.get("parent") in structs:
            fields[declaration["parent"]].append(declaration)

    lines = [
        "//! Generated safe Rust representations for plain framework structs.",
        "//!",
        "//! These are safe value representations, not promises of Objective-C ABI layout.",
        "",
        "use super::generated_value_types::*;",
        "use super::{Origin, Region};",
        "",
    ]
    for qualified_name in sorted(structs):
        name = qualified_name.rsplit("::", 1)[-1]
        special = SPECIAL_STRUCTS.get(qualified_name)
        if special:
            scalar, length = special
            components = PACKED_COMPONENTS[length]
            parameters = ", ".join(f"{component}: {scalar}" for component in components)
            values = ", ".join(components)
            lines += [
                f"/// Safe value representation for Metal `{qualified_name}`.",
                "#[derive(Clone, Copy, Debug, Default, PartialEq)]",
                f"pub struct {name}(pub [{scalar}; {length}]);",
                "",
                f"impl {name} {{",
                "    /// Creates a packed value from its individual components.",
                "    #[must_use]",
                f"    pub const fn new(values: [{scalar}; {length}]) -> Self {{",
                "        Self(values)",
                "    }",
                "",
                "    /// Creates a packed value from separate components.",
                "    #[must_use]",
                f"    pub const fn from_components({parameters}) -> Self {{",
                f"        Self([{values}])",
                "    }",
                "",
                "    /// Returns the packed components as an array.",
                "    #[must_use]",
                f"    pub const fn into_array(self) -> [{scalar}; {length}] {{",
                "        self.0",
                "    }",
                "}",
                "",
                f"impl std::ops::Index<usize> for {name} {{",
                f"    type Output = {scalar};",
                "",
                "    fn index(&self, index: usize) -> &Self::Output {",
                "        &self.0[index]",
                "    }",
                "}",
                "",
                f"impl std::ops::IndexMut<usize> for {name} {{",
                "    fn index_mut(&mut self, index: usize) -> &mut Self::Output {",
                "        &mut self.0[index]",
                "    }",
                "}",
                "",
            ]
            continue
        parsed_fields = [
            (field, *parse_field(field["signature"]))
            for field in sorted(fields[qualified_name], key=lambda item: item["ordinal"])
        ]
        lines += [
            f"/// Safe value representation for Metal `{qualified_name}`.",
            "#[derive(Clone, Debug, Default, PartialEq)]",
            f"pub struct {name} {{",
        ]
        for field, rust_name, field_type in parsed_fields:
            lines += [
                f"    /// Metal field `{field['qualified_name']}`.",
                f"    pub {rust_name}: {field_type},",
            ]
        lines += ["}", ""]
        if qualified_name == "MTL::PackedFloat4x3":
            lines += [
                "impl PackedFloat4x3 {",
                "    /// Creates a packed matrix from its four columns.",
                "    #[must_use]",
                "    pub const fn new(columns: [PackedFloat3; 4]) -> Self {",
                "        Self { columns }",
                "    }",
                "}",
                "",
                "impl std::ops::Index<usize> for PackedFloat4x3 {",
                "    type Output = PackedFloat3;",
                "",
                "    fn index(&self, index: usize) -> &Self::Output {",
                "        &self.columns[index]",
                "    }",
                "}",
                "",
                "impl std::ops::IndexMut<usize> for PackedFloat4x3 {",
                "    fn index_mut(&mut self, index: usize) -> &mut Self::Output {",
                "        &mut self.columns[index]",
                "    }",
                "}",
                "",
            ]
        if qualified_name == "NS::Range":
            lines += [
                "impl Range {",
                "    /// Creates a checked Foundation-style range.",
                "    #[must_use]",
                "    pub const fn new(location: usize, length: usize) -> Option<Self> {",
                "        if location.checked_add(length).is_some() {",
                "            Some(Self { location, length })",
                "        } else {",
                "            None",
                "        }",
                "    }",
                "",
                "    /// Returns whether the location is inside the half-open range.",
                "    #[must_use]",
                "    pub const fn contains(&self, location: usize) -> bool {",
                "        location >= self.location",
                "            && location < self.location.saturating_add(self.length)",
                "    }",
                "",
                "    /// Returns the exclusive range end when it does not overflow.",
                "    #[must_use]",
                "    pub const fn end(&self) -> Option<usize> {",
                "        self.location.checked_add(self.length)",
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
        raise RuntimeError(f"rustfmt failed for generated structs: {formatted.stderr}")
    return formatted.stdout


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--inventory", type=Path, default=Path("api/metal-cpp-inventory.json"))
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("crates/metal-rust-ffi/src/Metal/MTLGeneratedStructTypes.rs"),
    )
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()

    declarations = load_inventory(args.inventory)
    output = render(declarations)
    if args.check:
        if not args.output.is_file() or args.output.read_text(encoding="utf-8") != output:
            print(f"generated struct types are stale: {args.output}", file=sys.stderr)
            return 1
        print(f"generated struct types are current: {len(generated_struct_names(declarations))} types")
        return 0
    args.output.write_text(output, encoding="utf-8")
    print(f"generated {len(generated_struct_names(declarations))} safe struct types")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
