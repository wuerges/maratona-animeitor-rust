#!/usr/bin/env bash
set -euo pipefail
build_stage="startup"
trap 'printf "Vercel build failed during %s (line %s). See the error above.\n" "$build_stage" "$LINENO" >&2' ERR
cd "$(dirname "$0")/.."

# Validate configuration before installing tools or compiling the client.
config_file=$(mktemp)
trap 'rm -f "$config_file"' EXIT
build_stage="configuration validation"
echo "Validating Animeitor deployment configuration"
node scripts/vercel-config.mjs "$config_file"

build_stage="Rust toolchain setup"
echo "Preparing Rust nightly"
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
if ! command -v rustup >/dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain none
fi
rustup toolchain install nightly --profile minimal --target wasm32-unknown-unknown
# Trunk runs cargo metadata from the workspace root, which has no toolchain file.
# Fresh rustup installations above deliberately have no default toolchain.
export RUSTUP_TOOLCHAIN=nightly
build_stage="Trunk installation"
echo "Preparing Trunk"
if ! command -v trunk >/dev/null; then
  cargo +nightly install --locked trunk
fi
wasm_bindgen_version=$(awk -F'"' '/^name = "wasm-bindgen"$/{f=1} f && /^version = /{print $2; exit}' Cargo.lock)
build_stage="wasm-bindgen installation"
echo "Preparing wasm-bindgen $wasm_bindgen_version"
if ! command -v wasm-bindgen >/dev/null || [[ "$(wasm-bindgen --version)" != "wasm-bindgen $wasm_bindgen_version" ]]; then
  cargo +nightly install --locked --force "wasm-bindgen-cli@$wasm_bindgen_version"
fi

build_stage="frontend compilation"
echo "Building Animeitor frontend"
(cd animeitor-client && trunk build --locked --release --dist dist --public-url /)
build_stage="configuration output"
cp "$config_file" animeitor-client/dist/config.json
