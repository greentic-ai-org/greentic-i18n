#!/usr/bin/env bash
set -euo pipefail

echo "Installing Rust toolchain (stable)..."
if ! command -v rustup >/dev/null 2>&1; then
  curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
  source "$HOME/.cargo/env"
else
  rustup toolchain install stable
fi

export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
mkdir -p "$CARGO_HOME" "$RUSTUP_HOME"
echo "Caching directories: CARGO_HOME=$CARGO_HOME, RUSTUP_HOME=$RUSTUP_HOME"

export PATH="$HOME/.cargo/bin:$PATH"
rustup component add rustfmt clippy

echo "Running fmt/clippy/test in parallel..."
cargo fmt --all &
fmt_pid=$!
cargo clippy --workspace --all-targets &
clippy_pid=$!
cargo test --workspace &
test_pid=$!

wait "$fmt_pid"
wait "$clippy_pid"
wait "$test_pid"

if [[ -z "${CARGO_REGISTRY_TOKEN:-}" ]]; then
  echo "CARGO_REGISTRY_TOKEN is not set; aborting publish" >&2
  exit 1
fi

echo "Publishing to crates.io (token-protected)..."
MANIFESTS=(
  "crates/greentic-i18n-lib/Cargo.toml"
  "crates/greentic-i18n/Cargo.toml"
)

for manifest in "${MANIFESTS[@]}"; do
  cargo publish --token "$CARGO_REGISTRY_TOKEN" --manifest-path "$manifest"
done
