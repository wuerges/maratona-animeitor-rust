#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# Validate configuration before installing tools or compiling the client.
config_file=$(mktemp)
trap 'rm -f "$config_file"' EXIT
node scripts/vercel-config.mjs "$config_file"

export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
if ! command -v rustup >/dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain none
fi
rustup toolchain install nightly --profile minimal --target wasm32-unknown-unknown
if ! command -v trunk >/dev/null; then
  cargo +nightly install --locked trunk
fi
wasm_bindgen_version=$(awk -F'"' '/^name = "wasm-bindgen"$/{f=1} f && /^version = /{print $2; exit}' Cargo.lock)
if ! command -v wasm-bindgen >/dev/null || [[ "$(wasm-bindgen --version)" != "wasm-bindgen $wasm_bindgen_version" ]]; then
  cargo +nightly install --locked --force "wasm-bindgen-cli@$wasm_bindgen_version"
fi

(cd animeitor-client && trunk build --locked --release --dist dist --public-url /)
cp "$config_file" animeitor-client/dist/config.json
