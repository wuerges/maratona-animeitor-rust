FROM rust:slim-bookworm AS chef
RUN apt-get update && apt-get install -y musl-tools perl make ca-certificates \
 && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl \
 && rustup toolchain install nightly --target wasm32-unknown-unknown --profile minimal
WORKDIR /build

# wasm-bindgen-cli must match the version in Cargo.lock, otherwise trunk
# downloads its own copy (silently, and it can stall on this network).
COPY Cargo.lock ./
RUN cargo install --locked cargo-chef trunk \
 && cargo install --locked wasm-bindgen-cli --version \
      "$(awk -F'"' '/^name = "wasm-bindgen"$/{f=1} f && /^version = /{print $2; exit}' Cargo.lock)"

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
# Dependency-only builds, cached by Docker until a manifest/lock changes.
RUN cargo chef cook --release --recipe-path recipe.json \
      --target x86_64-unknown-linux-musl -p cli --features vendored \
 && RUSTUP_TOOLCHAIN=nightly cargo chef cook --release --recipe-path recipe.json \
      --target wasm32-unknown-unknown -p client-v2
COPY . .
RUN cargo build -p cli --release --target x86_64-unknown-linux-musl --features vendored \
 && cd client-v2 && trunk build --release -d release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/simples /simples
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/printurls /printurls
COPY --from=builder /build/client-v2/release /dist
ENTRYPOINT ["/simples"]
