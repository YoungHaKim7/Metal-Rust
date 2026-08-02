#!/usr/bin/env python3
"""Generate safe Rust aliases for non-callback metal-cpp aliases."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any

from generate_object_types import NAMESPACES, object_path
from generate_value_types import generated_type_names


PRIMITIVES = {
    "double": "f64",
    "NS::Integer": "isize",
    "NS::UInteger": "usize",
    "std::intptr_t": "isize",
    "std::uintptr_t": "usize",
    "std::uint64_t": "u64",
    "uint32_t": "u32",
    "uint64_t": "u64",
    "unsigned short": "u16",
}


def load_inventory(path: Path) -> list[dict[str, Any]]:
    value = json.loads(path.read_text(encoding="utf-8"))
    declarations = value.get("declarations")
    if not isinstance(declarations, list):
        raise ValueError("inventory.declarations must be a list")
    return declarations


def alias_target(declaration: dict[str, Any], declarations: list[dict[str, Any]]) -> str | None:
    qualified_name = declaration["qualified_name"]
    if qualified_name in generated_type_names(declarations):
        name = qualified_name.rsplit("::", 1)[-1]
        return f"super::super::generated_value_types::{name}"
    value = declaration.get("value")
    if value in PRIMITIVES:
        return PRIMITIVES[value]
    if value in {"class String*", "NS::String*"}:
        return "String"
    if isinstance(value, str) and value.endswith("*"):
        target = value.removeprefix("const ").removesuffix("*").strip()
        if target.startswith(tuple(f"{namespace}::" for namespace in NAMESPACES)):
            return f"super::super::generated_object_types::{object_path(target)}"
    if value == "MTL::SamplePosition":
        return "super::super::generated_struct_types::SamplePosition"
    return None


def generated_alias_paths(declarations: list[dict[str, Any]]) -> dict[str, str]:
    result = {}
    for declaration in declarations:
        if declaration["kind"] != "alias" or alias_target(declaration, declarations) is None:
            continue
        namespace, name = declaration["qualified_name"].split("::", 1)
        result[declaration["id"]] = (
            f"metal::generated_alias_types::{NAMESPACES[namespace]}::{name}"
        )
    return result


def render(declarations: list[dict[str, Any]]) -> str:
    aliases: dict[str, dict[str, tuple[str, str]]] = defaultdict(dict)
    for declaration in declarations:
        if declaration["kind"] != "alias" or "::" not in declaration["qualified_name"]:
            continue
        target = alias_target(declaration, declarations)
        if target is None:
            continue
        namespace, name = declaration["qualified_name"].split("::", 1)
        aliases[NAMESPACES[namespace]][name] = (declaration["qualified_name"], target)

    reverse = {module: namespace for namespace, module in NAMESPACES.items()}
    lines = [
        "//! Generated safe substitutions for non-callback C++ aliases.",
        "",
        "#![allow(non_camel_case_types)]",
        "",
    ]
    for module in sorted(aliases):
        lines += [f"/// Safe aliases from the `{reverse[module]}` namespace.", f"pub mod {module} {{"]
        for name, (qualified_name, target) in sorted(aliases[module].items()):
            lines += [
                f"    /// Safe substitution for `{qualified_name}`.",
                f"    pub type {name} = {target};",
            ]
        lines += ["}", ""]
    source = "\n".join(lines)
    formatted = subprocess.run(
        ["rustfmt", "--edition", "2024", "--emit", "stdout"],
        input=source,
        check=False,
        capture_output=True,
        text=True,
    )
    if formatted.returncode != 0:
        raise RuntimeError(f"rustfmt failed for generated aliases: {formatted.stderr}")
    return formatted.stdout


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--inventory", type=Path, default=Path("api/metal-cpp-inventory.json"))
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("crates/metal-rust-ffi/src/Metal/MTLGeneratedAliasTypes.rs"),
    )
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    declarations = load_inventory(args.inventory)
    output = render(declarations)
    count = len(set(generated_alias_paths(declarations).values()))
    if args.check:
        if not args.output.is_file() or args.output.read_text(encoding="utf-8") != output:
            print(f"generated alias types are stale: {args.output}", file=sys.stderr)
            return 1
        print(f"generated alias types are current: {count} aliases")
        return 0
    args.output.write_text(output, encoding="utf-8")
    print(f"generated {count} safe aliases")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
