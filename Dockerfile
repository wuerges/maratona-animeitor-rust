FROM rust:slim-bookworm AS builder

RUN apt-get update && apt-get install -y perl make ca-certificates \
 && rm -rf /var/lib/apt/lists/*
RUN rustup toolchain install nightly --target wasm32-unknown-unknown --profile minimal

# wasm-bindgen-cli must match the version in Cargo.lock, otherwise trunk
# downloads its own copy (silently, and it can stall on this network).
COPY Cargo.lock ./
RUN cargo install --locked trunk "wasm-bindgen-cli@$(awk -F'"' '/^name = "wasm-bindgen"$/{f=1} f && /^version = /{print $2; exit}' Cargo.lock)"

WORKDIR /build
COPY . .
# note: /build/target must NOT be a cache mount — later stages COPY from it
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build -p server-v2 -p cli --release \
 && cd client-v2 && trunk build --release -d release --public-url /animeitor/

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/simples /simples
COPY --from=builder /build/target/release/printurls /printurls
COPY --from=builder /build/target/release/update_contest_state /update_contest_state
COPY --from=builder /build/client-v2/release /dist
ENTRYPOINT ["/simples"]
