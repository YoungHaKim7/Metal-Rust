#!/usr/bin/env python3
"""Audit the safe facade and the publicly reachable FFI boundary.

This is intentionally a source audit, not a Rust parser.  The important
distinction is that Objective-C declarations in private FFI modules are
implementation details: only items re-exported from the FFI crate root are
checked for signature leaks.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path


UNSAFE_WORD = re.compile(r"\bunsafe\b")
UNSAFE_BLOCK = re.compile(r"\bunsafe\s*\{")
PUBLIC_UNSAFE_ITEM = re.compile(
    r"\bpub(?!\s*\()\s+(?:const\s+|async\s+)*unsafe\s+"
    r"(?:extern\s+(?:\"[^\"]+\"\s+)?fn|fn|trait)\b"
)
UNSAFE_EXTERN_BLOCK = re.compile(r"\bunsafe\s+extern(?:\s+\"[^\"]+\")?\s*\{")
PUBLIC_EXTERN_MEMBER = re.compile(r"\bpub(?!\s*\()\s+(?:safe\s+|unsafe\s+)?fn\b")
PUBLIC_DECLARATION = re.compile(
    r"\bpub(?!\s*\()\s+(?:(?:const|async|unsafe)\s+)*"
    r"(?:extern\s+(?:\"[^\"]+\"\s+)?fn|fn|struct|enum|union|type|trait)\b"
)
PUBLIC_FUNCTION = re.compile(
    r"\bpub(?!\s*\()\s+(?:(?:const|async|unsafe)\s+)*"
    r"(?:extern\s+(?:\"[^\"]+\"\s+)?)?fn\s+([A-Za-z_][A-Za-z0-9_]*)"
)
TYPE_DEFINITION = re.compile(
    r"\bpub(?!\s*\()\s+(?:unsafe\s+)?(?:struct|enum|union|type|trait)\s+"
    r"([A-Za-z_][A-Za-z0-9_]*)"
)
IMPL_START = re.compile(
    r"\bimpl(?:\s*<[^{};]*>)?\s+(?:[^{};]+\s+for\s+)?"
    r"(?:(?:crate|self|super)::[A-Za-z0-9_:]+::)?([A-Za-z_][A-Za-z0-9_]*)"
    r"(?:\s*<[^{};]*>)?\s*(?:where\s+[^{}]*)?\s*\{"
)
EXPLICIT_SEND_SYNC = re.compile(
    r"\bunsafe\s+impl(?:\s*<[^{};]*>)?\s+(Send|Sync)\s+for\s+"
    r"(?:(?:crate|self|super)::[A-Za-z0-9_:]+::)?([A-Za-z_][A-Za-z0-9_]*)"
)
PUBLIC_STRUCT = re.compile(
    r"\bpub(?!\s*\()\s+struct\s+([A-Za-z_][A-Za-z0-9_]*)[^;{]*\{"
)
PUBLIC_BRACED_TYPE = re.compile(
    r"\bpub(?!\s*\()\s+(?:unsafe\s+)?(struct|enum|union|trait)\s+"
    r"([A-Za-z_][A-Za-z0-9_]*)[^;{]*\{"
)
AUTO_TRAIT_SENSITIVE_FIELD = re.compile(
    r"\b(?:Retained|ProtocolObject|AnyObject|AnyClass)\b"
)
THREAD_BOUND_MARKER = re.compile(r"\b_thread_bound\s*:\s*(?:crate::)?ThreadBound\b")
THREAD_BOUND_DEFINITION = re.compile(
    r"\bstruct\s+ThreadBound\s*\(\s*PhantomData\s*<\s*Rc\s*<\s*\(\s*\)\s*>\s*>\s*\)"
)

FORBIDDEN_SIGNATURE_TYPES: tuple[tuple[str, re.Pattern[str]], ...] = (
    ("raw pointer", re.compile(r"\*(?:const|mut)\b")),
    ("objc2 type", re.compile(r"\bobjc2(?:::|\b)")),
    (
        "Objective-C implementation type",
        re.compile(
            r"\b(?:Retained|ProtocolObject|AnyObject|AnyClass|NSError|Sel|NonNull)\b"
        ),
    ),
)

# These names expose the C++/Objective-C ownership mechanism instead of a Rust
# value, Clone/Drop, or Iterator API.  Add safe functionality under canonical
# Rust names; do not add exceptions here.
GENERATED_LIFECYCLE_NAMES = re.compile(
    r"^(?:SharedPtr|TransferPtr|RetainPtr|AutoreleasePool|FastEnumeration|Referencing|"
    r"Autoreleased[A-Za-z0-9_]*)$"
)
GENERATED_LIFECYCLE_METHODS = re.compile(
    r"^(?:retain|release|autorelease|retain_count|retainCount|shared_ptr|sharedPtr)$"
)
GENERATED_PUBLIC_MODULE = re.compile(r"^generated(?:_[A-Za-z0-9_]+)?$")

# Explicit unsafe Send/Sync implementations are denied unless their two entries
# are reviewed here.  An entry must include a stable source location and a
# non-empty framework-backed proof.  This table does not claim anything about
# automatically derived traits; that is handled by --send-sync-policy.
SEND_SYNC_IMPL_ALLOWLIST: dict[tuple[str, str], tuple[str, str]] = {}


@dataclass(frozen=True)
class Finding:
    category: str
    path: Path
    line: int
    message: str

    def render(self) -> str:
        return f"{self.category}: {self.path}:{self.line}: {self.message}"


def rust_files(root: Path) -> list[Path]:
    return sorted(path for path in root.rglob("*.rs") if path.is_file())


def line_number(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def code_without_line_comments(text: str) -> str:
    """Blank line comments while retaining offsets and line numbers."""

    return "\n".join(line.split("//", 1)[0] for line in text.splitlines())


def matching_brace(text: str, opening: int) -> int | None:
    depth = 0
    for index in range(opening, len(text)):
        char = text[index]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return index
    return None


def declaration_end(text: str, start: int) -> int:
    """Return the end of an item signature, excluding its implementation body."""

    paren = bracket = angle = 0
    for index in range(start, len(text)):
        char = text[index]
        if char == "(":
            paren += 1
        elif char == ")":
            paren = max(0, paren - 1)
        elif char == "[":
            bracket += 1
        elif char == "]":
            bracket = max(0, bracket - 1)
        elif char == "<":
            angle += 1
        elif char == ">":
            angle = max(0, angle - 1)
        elif char in "{;" and paren == bracket == angle == 0:
            return index + 1
    return len(text)


def exported_ffi_items(lib_text: str) -> tuple[set[str], set[str]]:
    """Return explicit root re-export names and publicly exported modules."""

    names: set[str] = set()
    modules: set[str] = set()
    compact = code_without_line_comments(lib_text)
    for match in re.finditer(r"\bpub\s+use\s+([^;]+);", compact, re.DOTALL):
        clause = match.group(1).strip()
        if "{" in clause:
            body = clause.split("{", 1)[1].rsplit("}", 1)[0]
            for entry in body.split(","):
                entry = entry.strip()
                if not entry or entry == "self":
                    continue
                name = entry.split(" as ")[-1].strip().split("::")[-1]
                if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", name):
                    names.add(name)
            continue
        name = clause.split(" as ")[-1].strip().split("::")[-1]
        if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", name):
            names.add(name)
            modules.add(name)
    for _, declaration in public_declarations(lib_text):
        definition = TYPE_DEFINITION.search(declaration)
        function = PUBLIC_FUNCTION.search(declaration)
        name = definition.group(1) if definition else function.group(1) if function else None
        if name:
            names.add(name)
    return names, modules


def forbidden_signature_reason(signature: str) -> str | None:
    for description, pattern in FORBIDDEN_SIGNATURE_TYPES:
        if pattern.search(signature):
            return description
    return None


def public_declarations(text: str):
    code = code_without_line_comments(text)
    for match in PUBLIC_DECLARATION.finditer(code):
        end = declaration_end(code, match.start())
        yield match.start(), code[match.start() : end]


def public_braced_type_boundaries(text: str):
    code = code_without_line_comments(text)
    for match in PUBLIC_BRACED_TYPE.finditer(code):
        opening = code.find("{", match.start(), match.end())
        closing = matching_brace(code, opening)
        if closing is not None:
            yield match.group(1), match.group(2), match.start(), code[opening + 1 : closing]


def public_struct_field_fragments(body: str):
    """Yield externally visible named fields from a braced struct body."""

    start = 0
    paren = bracket = angle = brace = 0
    for index, char in enumerate(body):
        if char == "(":
            paren += 1
        elif char == ")":
            paren = max(0, paren - 1)
        elif char == "[":
            bracket += 1
        elif char == "]":
            bracket = max(0, bracket - 1)
        elif char == "<":
            angle += 1
        elif char == ">":
            angle = max(0, angle - 1)
        elif char == "{":
            brace += 1
        elif char == "}":
            brace = max(0, brace - 1)
        elif char == "," and paren == bracket == angle == brace == 0:
            fragment = body[start:index].strip()
            if re.match(r"^pub(?!\s*\()\s+", fragment):
                yield fragment
            start = index + 1
    fragment = body[start:].strip()
    if re.match(r"^pub(?!\s*\()\s+", fragment):
        yield fragment


def audit_facade(paths: list[Path]) -> list[Finding]:
    findings: list[Finding] = []
    for path in paths:
        text = path.read_text(encoding="utf-8")
        for number, line in enumerate(text.splitlines(), 1):
            if "#![forbid(unsafe_code)]" in line:
                continue
            code = line.split("//", 1)[0]
            if UNSAFE_WORD.search(code):
                findings.append(
                    Finding("facade-unsafe", path, number, "contains unsafe syntax")
                )
            if "objc2" in code:
                findings.append(
                    Finding("facade-objc2", path, number, "mentions an objc2 binding")
                )

        for offset, declaration in public_declarations(text):
            reason = forbidden_signature_reason(declaration)
            if reason:
                findings.append(
                    Finding(
                        "facade-signature",
                        path,
                        line_number(text, offset),
                        f"public declaration exposes {reason}",
                    )
                )
            definition = TYPE_DEFINITION.search(declaration)
            function = PUBLIC_FUNCTION.search(declaration)
            name = definition.group(1) if definition else function.group(1) if function else None
            if name and (
                GENERATED_LIFECYCLE_NAMES.fullmatch(name)
                or GENERATED_LIFECYCLE_METHODS.fullmatch(name)
            ):
                findings.append(
                    Finding(
                        "generated-lifecycle",
                        path,
                        line_number(text, offset),
                        f"public facade exposes generated ownership/helper shell `{name}`",
                    )
                )
        for kind, name, offset, body in public_braced_type_boundaries(text):
            fragments = public_struct_field_fragments(body) if kind == "struct" else (body,)
            for fragment in fragments:
                reason = forbidden_signature_reason(fragment)
                if reason:
                    findings.append(
                        Finding(
                            "facade-signature",
                            path,
                            line_number(text, offset),
                            f"public {kind} `{name}` exposes {reason}",
                        )
                    )
                    break
    return findings


def audit_unsafe_blocks(paths: list[Path]) -> list[Finding]:
    findings: list[Finding] = []
    for path in paths:
        lines = path.read_text(encoding="utf-8").splitlines()
        for index, line in enumerate(lines):
            if not UNSAFE_BLOCK.search(line):
                continue
            context = "\n".join(lines[max(0, index - 8) : index + 1])
            if "SAFETY:" not in context:
                findings.append(
                    Finding(
                        "ffi-unsafe-proof",
                        path,
                        index + 1,
                        "unsafe block lacks a nearby SAFETY comment",
                    )
                )
    return findings


def impl_bodies(text: str):
    code = code_without_line_comments(text)
    for match in IMPL_START.finditer(code):
        opening = code.find("{", match.start(), match.end())
        closing = matching_brace(code, opening)
        if closing is not None:
            yield match.group(1), match.start(), code[opening + 1 : closing], opening + 1


def audit_ffi_public_boundary(ffi_root: Path) -> list[Finding]:
    findings: list[Finding] = []
    lib_path = ffi_root / "lib.rs"
    lib_text = lib_path.read_text(encoding="utf-8")
    exported_names, exported_modules = exported_ffi_items(lib_text)
    marker_policy_used = False

    for module in sorted(exported_modules):
        if GENERATED_PUBLIC_MODULE.fullmatch(module):
            offset = lib_text.find(module)
            findings.append(
                Finding(
                    "generated-module",
                    lib_path,
                    line_number(lib_text, offset),
                    f"FFI crate publicly exposes generator layout `{module}`",
                )
            )

    for path in rust_files(ffi_root):
        text = path.read_text(encoding="utf-8")
        code = code_without_line_comments(text)

        # Opaque Objective-C owners must opt out of auto Send/Sync structurally.
        # Wrappers that only contain another already-marked wrapper inherit the
        # policy and do not need a redundant marker.
        for struct_match in PUBLIC_STRUCT.finditer(code):
            type_name = struct_match.group(1)
            if type_name not in exported_names:
                continue
            opening = code.find("{", struct_match.start(), struct_match.end())
            closing = matching_brace(code, opening)
            if closing is None:
                continue
            fields = code[opening + 1 : closing]
            if AUTO_TRAIT_SENSITIVE_FIELD.search(fields):
                marker_policy_used = True
            if AUTO_TRAIT_SENSITIVE_FIELD.search(fields) and not THREAD_BOUND_MARKER.search(fields):
                findings.append(
                    Finding(
                        "send-sync-marker",
                        path,
                        line_number(text, struct_match.start()),
                        f"exported Objective-C owner `{type_name}` lacks `_thread_bound: ThreadBound`",
                    )
                )

        for match in EXPLICIT_SEND_SYNC.finditer(code):
            trait_name, type_name = match.groups()
            entry = SEND_SYNC_IMPL_ALLOWLIST.get((type_name, trait_name))
            if entry is None:
                findings.append(
                    Finding(
                        "send-sync-impl",
                        path,
                        line_number(text, match.start()),
                        f"unsafe impl {trait_name} for {type_name} is not allowlisted",
                    )
                )
            elif entry[0] != str(path) or not entry[1].strip():
                findings.append(
                    Finding(
                        "send-sync-impl",
                        path,
                        line_number(text, match.start()),
                        f"allowlist entry for {type_name}: {trait_name} lacks an exact path/proof",
                    )
                )

        # Top-level exported types and free functions.
        for offset, declaration in public_declarations(text):
            definition = TYPE_DEFINITION.search(declaration)
            function = PUBLIC_FUNCTION.search(declaration)
            name = definition.group(1) if definition else function.group(1) if function else None
            if name not in exported_names:
                continue
            reason = forbidden_signature_reason(declaration)
            if reason:
                findings.append(
                    Finding(
                        "ffi-signature",
                        path,
                        line_number(text, offset),
                        f"exported `{name}` exposes {reason}",
                    )
                )
            if name and GENERATED_LIFECYCLE_NAMES.fullmatch(name):
                findings.append(
                    Finding(
                        "generated-lifecycle",
                        path,
                        line_number(text, offset),
                        f"FFI crate exposes generated ownership/helper shell `{name}`",
                    )
                )
            if PUBLIC_UNSAFE_ITEM.search(declaration):
                findings.append(
                    Finding(
                        "ffi-public-unsafe",
                        path,
                        line_number(text, offset),
                        f"exported `{name}` is unsafe",
                    )
                )

        for kind, name, offset, body in public_braced_type_boundaries(text):
            if name not in exported_names:
                continue
            fragments = public_struct_field_fragments(body) if kind == "struct" else (body,)
            for fragment in fragments:
                reason = forbidden_signature_reason(fragment)
                if reason:
                    findings.append(
                        Finding(
                            "ffi-signature",
                            path,
                            line_number(text, offset),
                            f"exported {kind} `{name}` exposes {reason}",
                        )
                    )
                    break

        # Public methods only matter when their receiver type is root-exported.
        for impl_name, impl_offset, body, body_offset in impl_bodies(text):
            if impl_name not in exported_names:
                continue
            for match in PUBLIC_FUNCTION.finditer(body):
                signature_end = declaration_end(body, match.start())
                signature = body[match.start() : signature_end]
                method_name = match.group(1)
                absolute = body_offset + match.start()
                reason = forbidden_signature_reason(signature)
                if reason:
                    findings.append(
                        Finding(
                            "ffi-signature",
                            path,
                            line_number(text, absolute),
                            f"{impl_name}::{method_name} exposes {reason}",
                        )
                    )
                if GENERATED_LIFECYCLE_METHODS.fullmatch(method_name):
                    findings.append(
                        Finding(
                            "generated-lifecycle",
                            path,
                            line_number(text, absolute),
                            f"{impl_name} exposes lifecycle method `{method_name}`",
                        )
                    )
                if PUBLIC_UNSAFE_ITEM.search(signature):
                    findings.append(
                        Finding(
                            "ffi-public-unsafe",
                            path,
                            line_number(text, absolute),
                            f"{impl_name}::{method_name} is unsafe",
                        )
                    )

        # An unsafe extern block is only a boundary violation when it occurs in
        # an exported type's impl. Private SDK declarations remain permitted.
        for impl_name, _, body, body_offset in impl_bodies(text):
            if impl_name not in exported_names:
                continue
            for extern_match in UNSAFE_EXTERN_BLOCK.finditer(body):
                opening = body.find("{", extern_match.start(), extern_match.end())
                closing = matching_brace(body, opening)
                extern_body = body[opening + 1 : closing if closing is not None else len(body)]
                for member in PUBLIC_EXTERN_MEMBER.finditer(extern_body):
                    absolute = body_offset + opening + 1 + member.start()
                    findings.append(
                        Finding(
                            "ffi-public-unsafe",
                            path,
                            line_number(text, absolute),
                            f"{impl_name} exposes a function from an unsafe extern block",
                        )
                    )

        # The same rule applies to a root declaration or to a private-module
        # extern member that is explicitly re-exported by name.
        for extern_match in UNSAFE_EXTERN_BLOCK.finditer(code):
            opening = code.find("{", extern_match.start(), extern_match.end())
            closing = matching_brace(code, opening)
            extern_body = code[opening + 1 : closing if closing is not None else len(code)]
            for member in PUBLIC_FUNCTION.finditer(extern_body):
                member_name = member.group(1)
                if path != lib_path and member_name not in exported_names:
                    continue
                absolute = opening + 1 + member.start()
                findings.append(
                    Finding(
                        "ffi-public-unsafe",
                        path,
                        line_number(text, absolute),
                        f"exported `{member_name}` is declared in an unsafe extern block",
                    )
                )
    if marker_policy_used and not THREAD_BOUND_DEFINITION.search(
        code_without_line_comments(lib_text)
    ):
        findings.append(
            Finding(
                "send-sync-marker",
                lib_path,
                1,
                "ThreadBound must contain PhantomData<Rc<()>> to deny both Send and Sync",
            )
        )
    return findings


def audit_send_sync_policy(policy_path: Path | None) -> list[Finding]:
    if policy_path is None:
        return []
    if not policy_path.is_file():
        return [
            Finding(
                "send-sync-policy",
                policy_path,
                1,
                "policy file is missing; automatic Send/Sync behavior was not audited",
            )
        ]
    try:
        policy = json.loads(policy_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        return [Finding("send-sync-policy", policy_path, 1, f"invalid policy: {error}")]
    if not isinstance(policy, dict) or policy.get("schema") != 1:
        return [
            Finding(
                "send-sync-policy",
                policy_path,
                1,
                "policy must be an object with schema=1",
            )
        ]
    entries = policy.get("types")
    if not isinstance(entries, list) or not entries:
        return [
            Finding(
                "send-sync-policy",
                policy_path,
                1,
                "policy has no audited public types",
            )
        ]
    findings: list[Finding] = []
    for index, entry in enumerate(entries):
        if not isinstance(entry, dict):
            findings.append(
                Finding("send-sync-policy", policy_path, 1, f"types[{index}] is not an object")
            )
            continue
        missing = {"type", "send", "sync", "evidence"} - entry.keys()
        if missing:
            findings.append(
                Finding(
                    "send-sync-policy",
                    policy_path,
                    1,
                    f"types[{index}] is missing {', '.join(sorted(missing))}",
                )
            )
            continue
        if not isinstance(entry["send"], bool) or not isinstance(entry["sync"], bool):
            findings.append(
                Finding(
                    "send-sync-policy",
                    policy_path,
                    1,
                    f"types[{index}] send/sync values must be booleans",
                )
            )
        if not isinstance(entry["evidence"], str) or not entry["evidence"].strip():
            findings.append(
                Finding(
                    "send-sync-policy",
                    policy_path,
                    1,
                    f"types[{index}] has no framework-backed evidence",
                )
            )
    return findings


def run_audit(root: Path, send_sync_policy: Path | None = None) -> list[Finding]:
    public_files = rust_files(root / "src") + rust_files(root / "tests")
    ffi_root = root / "crates" / "metal-rust-ffi" / "src"
    findings = audit_facade(public_files)
    findings.extend(audit_unsafe_blocks(rust_files(ffi_root)))
    findings.extend(audit_ffi_public_boundary(ffi_root))
    findings.extend(audit_send_sync_policy(send_sync_policy))
    return sorted(findings, key=lambda item: (str(item.path), item.line, item.category))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path("."))
    parser.add_argument(
        "--send-sync-policy",
        type=Path,
        help=(
            "validate a generated/maintained JSON policy for intentional public Send/Sync "
            "allowlisting; default !Send/!Sync opaque-owner markers are always audited"
        ),
    )
    args = parser.parse_args()
    root = args.root.resolve()
    policy = args.send_sync_policy
    if policy is not None and not policy.is_absolute():
        policy = root / policy
    findings = run_audit(root, policy)
    public_count = len(rust_files(root / "src")) + len(rust_files(root / "tests"))

    if findings:
        print(f"safety check failed with {len(findings)} error(s)", file=sys.stderr)
        for finding in findings:
            print(f"- {finding.render()}", file=sys.stderr)
        if policy is None:
            print(
                "note: opaque-owner !Send/!Sync markers were audited, but no intentional "
                "Send/Sync policy was supplied; pass --send-sync-policy <path> to validate one",
                file=sys.stderr,
            )
        return 1
    suffix = (
        "Send/Sync policy file validated; compile-time auto-trait assertions still required"
        if policy is not None
        else "opaque-owner !Send/!Sync markers audited; explicit Send/Sync policy NOT supplied"
    )
    print(f"safety check passed: {public_count} public Rust files audited; {suffix}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
