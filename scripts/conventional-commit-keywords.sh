#!/bin/sh

# This file is the single source of truth for Metal-Rust Conventional Commit
# keywords. Keep entries lowercase, unique, and separated by spaces.
# shellcheck disable=SC2034

METAL_RUST_CONVENTIONAL_COMMIT_TYPES='feat fix docs style refactor perf test build ci chore revert merge'

# Scopes are stable framework areas or repository concerns. Adding a scope
# requires updating this reviewed list before using it.
METAL_RUST_CONVENTIONAL_COMMIT_SCOPES='api capture counters deps ffi foundation metal metal4 metalfx package quartzcore release repo safety tests'
