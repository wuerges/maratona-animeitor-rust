FROM rust:1.68.2-bullseye AS build-base

RUN cargo install wasm-pack

FROM build-base AS build

WORKDIR /src
COPY . .

RUN wasm-pack build --dev --target web --out-name package client
RUN --mount=type=cache,mode=0777,target=/usr/local/cargo/registry \
    cargo build --release

FROM debian:buster-slim AS app

COPY --from=build /src/target/release/simples /simples

ARG HTTP_PORT
EXPOSE $HTTP_PORT

ENTRYPOINT ["/simples"]
