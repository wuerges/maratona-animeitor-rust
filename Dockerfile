FROM rust:slim-bookworm AS chef
RUN apt-get update && apt-get install -y musl-tools perl make ca-certificates \
 && rm -rf /var/lib/apt/lists/*
# wasm-bindgen-cli must match the version in Cargo.lock, otherwise trunk
# downloads its own copy (silently, and it can stall in this network).
RUN rustup target add x86_64-unknown-linux-musl \
 && rustup toolchain install nightly --target wasm32-unknown-unknown --profile minimal \
 && cargo install wasm-bindgen-cli --version 0.2.127 --locked \
 && cargo install cargo-chef --locked \
 && cargo install trunk --locked
WORKDIR /build

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
