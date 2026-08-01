#!/bin/sh
set -eu

script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
# The source path is resolved dynamically so the checker also works from hooks.
# shellcheck disable=SC1091
. "$script_dir/conventional-commit-keywords.sh"

usage() {
  printf '%s\n' \
    'usage: scripts/check-commit-message.sh --file PATH' \
    '       scripts/check-commit-message.sh --commit COMMIT' \
    '       scripts/check-commit-message.sh --subject SUBJECT' \
    '       scripts/check-commit-message.sh --self-test' >&2
  exit 2
}

run_self_test() {
  checker=$script_dir/check-commit-message.sh
  failures=0

  for allowed_type in $METAL_RUST_CONVENTIONAL_COMMIT_TYPES; do
    if ! "$checker" --subject "$allowed_type(repo): validate allowed type" >/dev/null 2>&1; then
      printf 'self-test rejected allowed type: %s\n' "$allowed_type" >&2
      failures=1
    fi
  done

  for allowed_scope in $METAL_RUST_CONVENTIONAL_COMMIT_SCOPES; do
    if ! "$checker" --subject "chore($allowed_scope): validate allowed scope" >/dev/null 2>&1; then
      printf 'self-test rejected allowed scope: %s\n' "$allowed_scope" >&2
      failures=1
    fi
  done

  while IFS= read -r invalid_subject; do
    if "$checker" --subject "$invalid_subject" >/dev/null 2>&1; then
      printf 'self-test accepted invalid subject: %s\n' "$invalid_subject" >&2
      failures=1
    fi
  done <<'EOF'
diag(metal): reject an unknown type
metal(repo): reject a framework used as type
test(test): reject a type used as scope
fix(unknown): reject an unknown scope
fix(metal,ffi): reject a compound scope
fix(metal/ffi): reject a compound scope variant
Fix(metal): reject a case variant
fix(Metal): reject a case variant
fix: reject a missing scope
fix(metal):
EOF

  if [ "$failures" -ne 0 ]; then
    return 1
  fi
  printf '%s\n' 'commit message policy self-test passed'
}

if [ "$#" -eq 1 ] && [ "$1" = '--self-test' ]; then
  run_self_test
  exit $?
fi

if [ "$#" -ne 2 ]; then
  usage
fi

case $1 in
  --file)
    if [ ! -f "$2" ]; then
      printf 'commit message file does not exist: %s\n' "$2" >&2
      exit 2
    fi
    subject=$(sed -n '1p' "$2")
    label=$2
    ;;
  --commit)
    if ! subject=$(git show -s --format=%s "$2" 2>/dev/null); then
      printf 'commit does not exist: %s\n' "$2" >&2
      exit 2
    fi
    label=$(git rev-parse --short "$2")
    ;;
  --subject)
    subject=$2
    label='provided subject'
    ;;
  *)
    usage
    ;;
esac

pattern='^[a-z][a-z0-9-]*\([a-z0-9][a-z0-9._/-]*\)!?: [^[:space:]].*$'

if ! printf '%s\n' "$subject" | LC_ALL=C grep -Eq "$pattern"; then
  printf 'invalid commit message (%s): %s\n' "$label" "$subject" >&2
  printf '%s\n' \
    'expected: type(scope): subject' \
    'example:  feat(metal): add safe command queue creation' >&2
  exit 1
fi

type=$(printf '%s\n' "$subject" | LC_ALL=C sed -E 's/^([a-z][a-z0-9-]*)\(.*/\1/')
scope=$(printf '%s\n' "$subject" | LC_ALL=C sed -E 's/^[a-z][a-z0-9-]*\(([a-z0-9][a-z0-9._\/-]*)\)!?:.*/\1/')

keyword_is_allowed() {
  keyword=$1
  allowed_keywords=$2
  for allowed_keyword in $allowed_keywords; do
    if [ "$keyword" = "$allowed_keyword" ]; then
      return 0
    fi
  done
  return 1
}

if ! keyword_is_allowed "$type" "$METAL_RUST_CONVENTIONAL_COMMIT_TYPES"; then
  printf 'invalid commit type (%s): %s\n' "$label" "$type" >&2
  printf 'allowed types: %s\n' "$METAL_RUST_CONVENTIONAL_COMMIT_TYPES" >&2
  exit 1
fi

if ! keyword_is_allowed "$scope" "$METAL_RUST_CONVENTIONAL_COMMIT_SCOPES"; then
  printf 'invalid commit scope (%s): %s\n' "$label" "$scope" >&2
  printf 'allowed scopes: %s\n' "$METAL_RUST_CONVENTIONAL_COMMIT_SCOPES" >&2
  exit 1
fi
