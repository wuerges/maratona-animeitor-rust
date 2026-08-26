FROM rust:slim-bookworm AS builder

RUN apt-get update && apt-get install -y musl-tools perl make ca-certificates \
 && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl \
 && rustup toolchain install nightly --target wasm32-unknown-unknown --profile minimal

# wasm-bindgen-cli must match the version in Cargo.lock, otherwise trunk
# downloads its own copy (silently, and it can stall on this network).
COPY Cargo.lock ./
RUN cargo install --locked trunk "wasm-bindgen-cli@$(awk -F'"' '/^name = "wasm-bindgen"$/{f=1} f && /^version = /{print $2; exit}' Cargo.lock)"

# rustc >= 1.98 links musl dynamically by default; the runtime image has no
# musl loader, so keep the binaries static.
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C target-feature=+crt-static"

WORKDIR /build
COPY . .
# note: /build/target must NOT be a cache mount — later stages COPY from it
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build -p cli --release --target x86_64-unknown-linux-musl --features vendored \
 && cd client-v2 && trunk build --release -d release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/simples /simples
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/printurls /printurls
COPY --from=builder /build/client-v2/release /dist
ENTRYPOINT ["/simples"]
