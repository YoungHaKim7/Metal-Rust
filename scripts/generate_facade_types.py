#!/usr/bin/env python3
"""Generate canonical safe facade types over the private FFI implementation."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any

from generate_object_types import (
    NAMESPACES,
    MATRIX_PROPERTIES,
    default_constructible_classes,
    generated_bool_setters,
    generated_command_buffer_encoders,
    generated_metal4_command_buffer_encoders,
    generated_metalfx_compiler_factories,
    generated_enum_properties,
    generated_object_properties,
    generated_property_methods,
    generated_static_device_queries,
    property_setter,
    snake_case,
)
from generate_struct_types import generated_struct_names
from generate_value_types import MANUAL_TYPES, generated_type_names


MANUAL_OBJECTS = {
    "CA::MetalDrawable",
    "CA::MetalLayer",
    "MTL4::CounterHeap",
    # Metal 4 command recording is represented exclusively by the canonical
    # typestate wrappers in `src/Metal4/MTL4Command.rs`.  Publishing these
    # cloneable generated shells would let callers bypass begin/end and the
    # exclusive encoder borrow.
    "MTL4::CommandBuffer",
    "MTL4::CommandEncoder",
    "MTL4::ComputeCommandEncoder",
    "MTL4::MachineLearningCommandEncoder",
    "MTL4::RenderCommandEncoder",
    "MTL::BlitCommandEncoder",
    "MTL::Buffer",
    "MTL::CaptureManager",
    "MTL::CommandBuffer",
    "MTL::CommandQueue",
    "MTL::CompileOptions",
    "MTL::ComputeCommandEncoder",
    "MTL::ComputePassDescriptor",
    "MTL::ComputePipelineState",
    "MTL::Device",
    "MTL::Function",
    "MTL::Library",
    "MTL::RenderCommandEncoder",
    "MTL::RenderPassDescriptor",
    "MTL::RenderPipelineDescriptor",
    "MTL::RenderPipelineState",
    "MTL::Texture",
    "MTL::TextureDescriptor",
    "MTLFX::SpatialScaler",
    "MTLFX::SpatialScalerDescriptor",
    "MTLFX::TemporalScaler",
    "MTLFX::TemporalScalerDescriptor",
}

BRIDGED_MANUAL_OBJECTS = {
    "MTL::Buffer",
    "MTL::CompileOptions",
    "MTL::Device",
    "MTL::Function",
    "MTL::Library",
    "MTL::Texture",
}

MANUAL_NAMES = {
    "foundation": {
        "DeviceCertification", "ErrorDomain", "ErrorUserInfoKey", "NotificationName",
        "Number", "ProcessPerformanceProfile",
    },
    "metal": {
        "ClearColor", "CommandBufferStatus", "CommonCounter", "CommonCounterSet",
        "DeviceNotificationName", "Origin", "PixelFormat", "PrimitiveType",
        "Region", "ResourceOptions", "Size", "StorageMode", "TextureType",
        "TextureUsage", "Viewport",
    },
    "metal_fx": {"SpatialScalerColorProcessingMode"},
}

PRIMITIVE_ALIASES = {
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

# Metal-CPP uses these aliases to describe autoreleased Objective-C out
# parameters. Safe Rust factories return owned RAII facade values directly, so
# publishing pointer-lifetime aliases would recreate an ownership mechanism
# that callers neither need nor can use safely.
OMITTED_OWNERSHIP_ALIASES = {
    "MTL::AutoreleasedArgument",
    "MTL::AutoreleasedComputePipelineReflection",
    "MTL::AutoreleasedRenderPipelineReflection",
}


def load_inventory(path: Path) -> list[dict[str, Any]]:
    value = json.loads(path.read_text(encoding="utf-8"))
    declarations = value.get("declarations")
    if not isinstance(declarations, list):
        raise ValueError("inventory.declarations must be a list")
    return declarations


def module_for(qualified_name: str) -> str:
    return NAMESPACES[qualified_name.split("::", 1)[0]]


def ffi_object(qualified_name: str) -> str:
    namespace, name = qualified_name.split("::", 1)
    return f"metal_rust_ffi::__private::objects::{NAMESPACES[namespace]}::{name}"


def public_type(qualified_name: str) -> str:
    return f"crate::{module_for(qualified_name)}::{qualified_name.rsplit('::', 1)[-1]}"


def available_object_names(declarations: list[dict[str, Any]]) -> set[str]:
    """Return object classes that have generated public facade wrappers."""
    return {
        item["qualified_name"]
        for item in declarations
        if item["kind"] == "class"
        and "::" in item["qualified_name"]
        and not item["qualified_name"].startswith("NS::")
        and item["qualified_name"] not in MANUAL_OBJECTS
    }


def generated_facade_method_paths(declarations: list[dict[str, Any]]) -> dict[str, str]:
    """Map only methods actually emitted on canonical facade wrappers."""
    available = available_object_names(declarations)
    constructors = default_constructible_classes(declarations)
    primitive = generated_property_methods(declarations)
    enum = generated_enum_properties(declarations)
    objects = generated_object_properties(declarations)
    boolean = generated_bool_setters(declarations)
    command_buffer_encoders = generated_command_buffer_encoders(declarations)
    metal4_command_buffer_encoders = generated_metal4_command_buffer_encoders(declarations)
    static_device_queries = generated_static_device_queries(declarations)
    metalfx_compiler_factories = generated_metalfx_compiler_factories(declarations)
    emitted: dict[tuple[str, str], str] = {}
    for parent in available:
        prefix = f"{module_for(parent)}::{parent.rsplit('::', 1)[-1]}"
        if parent in constructors:
            emitted[(parent, "alloc")] = f"{prefix}::new"
            emitted[(parent, "init")] = f"{prefix}::new"
        for selector, rust_name, rust_type, writable in primitive.get(parent, []):
            emitted[(parent, selector)] = f"{prefix}::{rust_name}"
            if writable and rust_type != "bool":
                setter = property_setter(parent, selector)
                emitted[(parent, setter)] = f"{prefix}::set_{rust_name}"
        for selector, rust_name, _type_name, _raw, writable in enum.get(parent, []):
            emitted[(parent, selector)] = f"{prefix}::{rust_name}"
            if writable:
                setter = property_setter(parent, selector)
                emitted[(parent, setter)] = f"{prefix}::{snake_case(setter)}"
        for selector, rust_name, target, writable in objects.get(parent, []):
            if target not in available and target not in BRIDGED_MANUAL_OBJECTS and target != "NS::String":
                continue
            emitted[(parent, selector)] = f"{prefix}::{rust_name}"
            if writable:
                setter = property_setter(parent, selector)
                emitted[(parent, setter)] = f"{prefix}::set_{rust_name}"
        for selector, rust_name in boolean.get(parent, []):
            emitted[(parent, selector)] = f"{prefix}::{rust_name}"
        if parent in command_buffer_encoders:
            emitted[(parent, "encodeToCommandBuffer")] = f"{prefix}::encode_to_command_buffer"
        if parent in metal4_command_buffer_encoders:
            emitted[(parent, "encodeToCommandBuffer")] = f"{prefix}::encode_to_command_buffer"
        for selector_name, rust_name, _target in metalfx_compiler_factories.get(parent, []):
            emitted[(parent, selector_name)] = f"{prefix}::{rust_name}"
        for selector_name, rust_name, _return_type, _objc_selector in static_device_queries.get(parent, []):
            emitted[(parent, selector_name)] = f"{prefix}::{rust_name}"
        for selector, rust_name, setter, _helper_get, _helper_set in MATRIX_PROPERTIES.get(parent, []):
            emitted[(parent, selector)] = f"{prefix}::{rust_name}"
            emitted[(parent, setter)] = f"{prefix}::set_{rust_name}"
    result: dict[str, str] = {}
    for item in declarations:
        key = (item.get("parent", ""), item["name"])
        if item["kind"] != "method" or key not in emitted:
            continue
        if item["name"] in {"alloc", "init"}:
            arguments = item["signature"].split("(", 1)[1].rsplit(")", 1)[0].strip()
            if arguments:
                # A generated `new()` only covers the zero-argument initializer;
                # same-name Metal-CPP overloads need their own safe translation.
                continue
        result[item["id"]] = emitted[key]
    return result


def generated_facade_alias_paths(declarations: list[dict[str, Any]]) -> dict[str, str]:
    """Map aliases that are actually emitted at a canonical public path."""
    available = available_object_names(declarations)
    values = generated_type_names(declarations)
    occupied: dict[str, set[str]] = defaultdict(set)
    for qualified_name in values | generated_struct_names(declarations) | available:
        module = module_for(qualified_name)
        occupied[module].add(qualified_name.rsplit("::", 1)[-1])
    for module, names in MANUAL_NAMES.items():
        occupied[module].update(names)
    result: dict[str, str] = {}
    for item in declarations:
        if item["kind"] != "alias" or "::" not in item["qualified_name"]:
            continue
        module = module_for(item["qualified_name"])
        name = item["qualified_name"].rsplit("::", 1)[-1]
        target = canonical_alias_target(item, available, values)
        if target is not None:
            # A metal-cpp alias may intentionally have the same name as the
            # canonical enum/options type. That declaration is still covered
            # by the existing public symbol even though no second Rust alias
            # can or should be emitted.
            result[item["id"]] = f"{module}::{name}"
            occupied[module].add(name)
    return result


def canonical_alias_target(
    declaration: dict[str, Any], available_objects: set[str], values: set[str]
) -> str | None:
    qualified_name = declaration["qualified_name"]
    if qualified_name in OMITTED_OWNERSHIP_ALIASES:
        return None
    if qualified_name in values:
        return public_type(qualified_name)
    value = declaration.get("value")
    if value in PRIMITIVE_ALIASES:
        return PRIMITIVE_ALIASES[value]
    if value in {"class String*", "NS::String*"}:
        return "String"
    if isinstance(value, str) and value.endswith("*"):
        target = value.removeprefix("const ").removesuffix("*").strip()
        if target in available_objects:
            return public_type(target)
    if value == "MTL::SamplePosition":
        return "crate::metal::SamplePosition"
    return None


def render(declarations: list[dict[str, Any]]) -> str:
    values = generated_type_names(declarations)
    enum_modules = {
        item["qualified_name"].rsplit("::", 1)[-1]: module_for(item["qualified_name"])
        for item in declarations
        if item["kind"] in {"enum", "options"} and "::" in item["qualified_name"]
    }
    structs = generated_struct_names(declarations)
    # Foundation collections and helper objects are represented by Rust-native
    # values in the facade instead of mirroring the Objective-C class hierarchy.
    available_objects = available_object_names(declarations)
    constructors = default_constructible_classes(declarations)
    primitives = generated_property_methods(declarations)
    enums = generated_enum_properties(declarations)
    objects = generated_object_properties(declarations)
    bool_setters = generated_bool_setters(declarations)
    command_buffer_encoders = generated_command_buffer_encoders(declarations)
    metal4_command_buffer_encoders = generated_metal4_command_buffer_encoders(declarations)
    static_device_queries = generated_static_device_queries(declarations)
    metalfx_compiler_factories = generated_metalfx_compiler_factories(declarations)

    module_values: dict[str, list[str]] = defaultdict(list)
    module_structs: dict[str, list[str]] = defaultdict(list)
    module_objects: dict[str, list[str]] = defaultdict(list)
    module_aliases: dict[str, list[tuple[str, str, str]]] = defaultdict(list)

    for qualified_name in values:
        module = module_for(qualified_name)
        name = qualified_name.rsplit("::", 1)[-1]
        if qualified_name not in MANUAL_TYPES and name not in MANUAL_NAMES.get(module, set()):
            module_values[module].append(name)
    for qualified_name in structs:
        module = module_for(qualified_name)
        name = qualified_name.rsplit("::", 1)[-1]
        if name not in MANUAL_NAMES.get(module, set()):
            module_structs[module].append(name)
    for qualified_name in available_objects:
        module_objects[module_for(qualified_name)].append(qualified_name)

    occupied = defaultdict(set)
    for module, names in module_values.items():
        occupied[module].update(names)
    for module, names in module_structs.items():
        occupied[module].update(names)
    for module, names in module_objects.items():
        occupied[module].update(name.rsplit("::", 1)[-1] for name in names)
    for module, names in MANUAL_NAMES.items():
        occupied[module].update(names)
    for item in declarations:
        if item["kind"] != "alias" or "::" not in item["qualified_name"]:
            continue
        module = module_for(item["qualified_name"])
        name = item["qualified_name"].rsplit("::", 1)[-1]
        target = canonical_alias_target(item, available_objects, values)
        if target is not None and name not in occupied[module]:
            module_aliases[module].append((name, item["qualified_name"], target))
            occupied[module].add(name)

    modules = sorted(set(module_values) | set(module_structs) | set(module_objects) | set(module_aliases))
    lines = [
        "//! Generated canonical facade wrappers.",
        "//!",
        "//! This module is private; framework modules re-export its safe types.",
        "",
        "#![allow(non_camel_case_types)]",
        "#![allow(dead_code)]",
        "",
    ]
    for module in modules:
        lines += [f"/// Safe generated implementation for the `{module}` canonical module.", f"pub(crate) mod {module} {{"]
        for name in sorted(set(module_values[module])):
            lines += [
                f"    /// Canonical safe value type `{name}`.",
                f"    pub use metal_rust_ffi::__private::values::{name};",
            ]
        for name in sorted(set(module_structs[module])):
            lines += [
                f"    /// Canonical safe structure `{name}`.",
                f"    pub use metal_rust_ffi::__private::structs::{name};",
            ]
        for name, qualified_name, target in sorted(module_aliases[module]):
            lines += [
                f"    /// Safe substitution for `{qualified_name}`.",
                f"    pub type {name} = {target};",
            ]
        for qualified_name in sorted(module_objects[module]):
            name = qualified_name.rsplit("::", 1)[-1]
            ffi = ffi_object(qualified_name)
            lines += [
                f"    /// Safe owned facade for `{qualified_name}`.",
                "    #[derive(Clone)]",
                f"    pub struct {name} {{",
                f"        pub(crate) inner: {ffi},",
                "    }",
                "",
                f"    impl {name} {{",
                f"        pub(crate) const fn from_ffi(inner: {ffi}) -> Self {{",
                "            Self { inner }",
                "        }",
            ]
            if qualified_name in constructors:
                lines += [
                    f"        /// Creates a default `{qualified_name}` after availability checks.",
                    "        pub fn new() -> Result<Self, crate::Error> {",
                    f"            {ffi}::new()",
                    "                .map(Self::from_ffi)",
                    "                .map_err(crate::Error::from_ffi)",
                    "        }",
                ]
            for _selector, rust_name, rust_type, writable in sorted(set(primitives.get(qualified_name, []))):
                lines += [
                    f"        /// Reads the `{rust_name}` property with availability checks.",
                    f"        pub fn {rust_name}(&self) -> Result<{rust_type}, crate::Error> {{",
                    f"            self.inner.{rust_name}().map_err(crate::Error::from_ffi)",
                    "        }",
                ]
                if writable and rust_type != "bool":
                    lines += [
                        f"        /// Writes the `{rust_name}` property after validation.",
                        f"        pub fn set_{rust_name}(&self, value: {rust_type}) -> Result<(), crate::Error> {{",
                        f"            self.inner.set_{rust_name}(value).map_err(crate::Error::from_ffi)",
                        "        }",
                    ]
            for selector, rust_name, type_name, _raw, writable in sorted(set(enums.get(qualified_name, []))):
                type_module = enum_modules[type_name]
                lines += [
                    f"        /// Reads the `{rust_name}` property with availability checks.",
                    f"        pub fn {rust_name}(&self) -> Result<crate::{type_module}::{type_name}, crate::Error> {{",
                    f"            self.inner.{rust_name}().map_err(crate::Error::from_ffi)",
                    "        }",
                ]
                if writable:
                    setter = snake_case(property_setter(qualified_name, selector))
                    lines += [
                        f"        /// Writes the `{rust_name}` property after validation.",
                        f"        pub fn {setter}(&self, value: crate::{type_module}::{type_name}) -> Result<(), crate::Error> {{",
                        f"            self.inner.{setter}(value).map_err(crate::Error::from_ffi)",
                        "        }",
                    ]
            for _selector, rust_name, target, writable in sorted(set(objects.get(qualified_name, []))):
                if target == "NS::String":
                    lines += [
                        f"        /// Reads the optional `{rust_name}` string property.",
                        f"        pub fn {rust_name}(&self) -> Result<Option<String>, crate::Error> {{",
                        f"            self.inner.{rust_name}().map_err(crate::Error::from_ffi)",
                        "        }",
                    ]
                    if writable:
                        lines += [
                            f"        /// Writes the `{rust_name}` string property.",
                            f"        pub fn set_{rust_name}(&self, value: &str) -> Result<(), crate::Error> {{",
                            f"            self.inner.set_{rust_name}(value).map_err(crate::Error::from_ffi)",
                            "        }",
                        ]
                    continue
                if target not in available_objects and target not in BRIDGED_MANUAL_OBJECTS:
                    continue
                target_type = public_type(target)
                lines += [
                    f"        /// Reads the optional `{rust_name}` object property.",
                    f"        pub fn {rust_name}(&self) -> Result<Option<{target_type}>, crate::Error> {{",
                    f"            self.inner.{rust_name}()",
                    f"                .map(|value| value.map({target_type}::from_ffi))",
                    "                .map_err(crate::Error::from_ffi)",
                    "        }",
                ]
                if writable:
                    lines += [
                        f"        /// Writes the `{rust_name}` object property.",
                        f"        pub fn set_{rust_name}(&self, value: Option<&{target_type}>) -> Result<(), crate::Error> {{",
                        f"            self.inner.set_{rust_name}(value.map(|value| &value.inner)).map_err(crate::Error::from_ffi)",
                        "        }",
                    ]
            for _selector, rust_name in sorted(set(bool_setters.get(qualified_name, []))):
                lines += [
                    "        /// Writes the Boolean property after availability checks.",
                    f"        pub fn {rust_name}(&self, value: bool) -> Result<(), crate::Error> {{",
                    f"            self.inner.{rust_name}(value).map_err(crate::Error::from_ffi)",
                    "        }",
                ]
            if qualified_name in command_buffer_encoders:
                lines += [
                    "        /// Encodes work into a mutably borrowed Metal command buffer.",
                    "        pub fn encode_to_command_buffer(&self, command_buffer: &mut crate::metal::CommandBuffer) -> Result<(), crate::Error> {",
                    "            self.inner.encode_to_command_buffer(&mut command_buffer.inner).map_err(crate::Error::from_ffi)",
                    "        }",
                ]
            if qualified_name in metal4_command_buffer_encoders:
                recording_method = f"encode_{snake_case(qualified_name.rsplit('::', 1)[-1])}"
                lines += [
                    "        /// Encodes work into an active Metal 4 recording scope.",
                    "        pub fn encode_to_command_buffer(&self, command_buffer: &mut crate::metal4::RecordingCommandBuffer) -> Result<(), crate::Error> {",
                    f"            command_buffer.as_ffi_mut().{recording_method}(&self.inner).map_err(crate::Error::from_ffi)",
                    "        }",
                ]
            for _selector_name, rust_name, target in sorted(
                set(metalfx_compiler_factories.get(qualified_name, []))
            ):
                target_type = public_type(target)
                lines += [
                    f"        /// Creates a `{target}` from a device and Metal 4 compiler.",
                    f"        pub fn {rust_name}(&self, device: &crate::metal::Device, compiler: &crate::metal4::Compiler) -> Result<{target_type}, crate::Error> {{",
                    f"            self.inner.{rust_name}(&device.inner, &compiler.inner).map({target_type}::from_ffi).map_err(crate::Error::from_ffi)",
                    "        }",
                ]
            for _selector_name, rust_name, return_type, _objc_selector in sorted(
                set(static_device_queries.get(qualified_name, []))
            ):
                lines += [
                    "        /// Runs a static MetalFX device query after availability checks.",
                    f"        pub fn {rust_name}(device: &crate::metal::Device) -> Result<{return_type}, crate::Error> {{",
                    f"            {ffi}::{rust_name}(&device.inner).map_err(crate::Error::from_ffi)",
                    "        }",
                ]
            for _selector, rust_name, _setter, _helper_get, _helper_set in MATRIX_PROPERTIES.get(
                qualified_name, []
            ):
                lines += [
                    "        /// Reads an aligned MetalFX transformation matrix.",
                    f"        pub fn {rust_name}(&self) -> Result<crate::metal_fx::Matrix4x4, crate::Error> {{",
                    f"            self.inner.{rust_name}().map_err(crate::Error::from_ffi)",
                    "        }",
                    "        /// Writes a finite aligned MetalFX transformation matrix.",
                    f"        pub fn set_{rust_name}(&self, value: crate::metal_fx::Matrix4x4) -> Result<(), crate::Error> {{",
                    f"            self.inner.set_{rust_name}(value).map_err(crate::Error::from_ffi)",
                    "        }",
                ]
            lines += ["    }", ""]
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
        raise RuntimeError(f"rustfmt failed for generated facade: {formatted.stderr}")
    return formatted.stdout


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--inventory", type=Path, default=Path("api/metal-cpp-inventory.json"))
    parser.add_argument("--output", type=Path, default=Path("src/Private/GeneratedFacade.rs"))
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    output = render(load_inventory(args.inventory))
    if args.check:
        if not args.output.is_file() or args.output.read_text(encoding="utf-8") != output:
            print(f"generated facade is stale: {args.output}", file=sys.stderr)
            return 1
        print("generated facade is current")
        return 0
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(output, encoding="utf-8")
    print(f"generated canonical facade: {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
