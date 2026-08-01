#!/bin/sh
set -eu

repo_root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
git -C "$repo_root" config core.hooksPath .githooks
printf '%s\n' 'installed Metal-Rust Git hooks via core.hooksPath=.githooks'
