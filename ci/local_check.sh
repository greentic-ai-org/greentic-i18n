#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

step() {
  echo "Running: $*"
  "$@"
}

step cargo fmt --all
step cargo clippy --workspace --all-targets -- -D warnings
step cargo test --workspace
MANIFESTS=(
  "crates/greentic-i18n-lib/Cargo.toml"
  "crates/greentic-i18n/Cargo.toml"
)

for manifest in "${MANIFESTS[@]}"; do
  cargo publish --dry-run --manifest-path "$manifest" --allow-dirty
done
