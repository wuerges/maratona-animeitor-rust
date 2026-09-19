FROM rust:slim-bookworm AS builder

RUN apt-get update && apt-get install -y perl make ca-certificates \
 && rm -rf /var/lib/apt/lists/*
RUN rustup toolchain install nightly --target wasm32-unknown-unknown --profile minimal

# wasm-bindgen-cli must match the version in Cargo.lock, otherwise trunk
# downloads its own copy (silently, and it can stall on this network).
COPY Cargo.lock ./
RUN cargo install --locked trunk "wasm-bindgen-cli@$(awk -F'"' '/^name = "wasm-bindgen"$/{f=1} f && /^version = /{print $2; exit}' Cargo.lock)"

RUN apt-get update && apt-get install -y pkg-config libssl-dev \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .
# Fail early if the installed tool differs from the client dependency. Trunk's
# offline mode also prevents its own wasm-bindgen download fallback.
RUN test "$(wasm-bindgen --version)" = "wasm-bindgen $(awk -F'"' '/^name = "wasm-bindgen"$/{f=1} f && /^version = /{print $2; exit}' Cargo.lock)"

# Fetch explicitly: Trunk captures cargo metadata output while it resolves all
# workspace dependencies, which can otherwise look like a hung build.
# note: /build/target must NOT be a cache mount — later stages COPY from it
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo fetch --locked \
 && cargo build -p server-v2 -p cli --release --locked \
 && cd animeitor-client && CARGO_NET_OFFLINE=true trunk build --offline --locked --release -d release --public-url /animeitor/

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/animeitor-server /animeitor-server
COPY --from=builder /build/target/release/printurls /printurls
COPY --from=builder /build/target/release/animeitor-admin /animeitor-admin
COPY --from=builder /build/target/release/animeitor-feeder /animeitor-feeder
COPY --from=builder /build/animeitor-client/release /dist
ENTRYPOINT ["/animeitor-server"]
