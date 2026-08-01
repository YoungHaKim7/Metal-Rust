#!/bin/sh
set -eu

usage() {
  printf '%s\n' \
    'usage: scripts/check-release-environment.sh MIN_VERSION MAX_EXCLUSIVE' >&2
  exit 2
}

if [ "$#" -ne 2 ]; then
  usage
fi

minimum_version=$1
maximum_exclusive=$2

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'required release command is unavailable: %s\n' "$1" >&2
    exit 1
  fi
}

for command_name in cargo git python3 rustc rustfmt sw_vers xcode-select xcrun; do
  require_command "$command_name"
done

if [ "$(uname -s)" != 'Darwin' ]; then
  printf 'release runner must be macOS; found: %s\n' "$(uname -s)" >&2
  exit 1
fi

host_arch=$(uname -m)
case $host_arch in
  arm64 | x86_64) ;;
  *)
    printf 'unsupported release runner architecture: %s\n' "$host_arch" >&2
    exit 1
    ;;
esac

rust_version=$(rustc --version --verbose | sed -n 's/^release: //p')
cargo_version=$(cargo --version | awk 'NR == 1 { print $2 }')

python3 - "$minimum_version" "$maximum_exclusive" "$rust_version" "$cargo_version" <<'PY'
import re
import sys


def parse_version(label, value):
    match = re.fullmatch(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:[-+].*)?", value)
    if match is None:
        raise SystemExit(f"unable to parse {label} version: {value!r}")
    return tuple(int(component) for component in match.groups())


minimum = parse_version("minimum", sys.argv[1])
maximum = parse_version("maximum", sys.argv[2])
rust = parse_version("rustc", sys.argv[3])
cargo = parse_version("cargo", sys.argv[4])

for label, version in (("rustc", rust), ("cargo", cargo)):
    if not minimum <= version < maximum:
        raise SystemExit(
            f"{label} {version!s} is outside the supported range "
            f">={sys.argv[1]}, <{sys.argv[2]}"
        )

if rust[:2] != cargo[:2]:
    raise SystemExit(
        f"cargo {sys.argv[4]} and rustc {sys.argv[3]} must share a major/minor toolchain"
    )
PY

if ! cargo fmt --version >/dev/null 2>&1; then
  printf '%s\n' 'cargo fmt is unavailable for the active toolchain' >&2
  exit 1
fi
if ! cargo clippy --version >/dev/null 2>&1; then
  printf '%s\n' 'cargo clippy is unavailable for the active toolchain' >&2
  exit 1
fi

xcode_path=$(xcode-select -p)
sdk_path=$(xcrun --sdk macosx --show-sdk-path)
clang_path=$(xcrun --sdk macosx --find clang)

for required_path in "$xcode_path" "$sdk_path" "$clang_path"; do
  if [ ! -e "$required_path" ]; then
    printf 'release environment path does not exist: %s\n' "$required_path" >&2
    exit 1
  fi
done

printf '%s\n' \
  "macOS: $(sw_vers -productVersion) ($(sw_vers -buildVersion))" \
  "architecture: $host_arch" \
  "rustc: $(command -v rustc) ($rust_version)" \
  "cargo: $(command -v cargo) ($cargo_version)" \
  "rustfmt: $(rustfmt --version)" \
  "clippy: $(cargo clippy --version)" \
  "Xcode: $xcode_path" \
  "macOS SDK: $sdk_path" \
  "clang: $clang_path"
