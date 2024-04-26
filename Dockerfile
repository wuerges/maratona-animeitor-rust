FROM rust:1 AS build-client

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk

WORKDIR /src
COPY client-v2 client-v2
COPY server/data server/data
COPY server/Cargo.toml server

ARG URL_PREFIX
RUN cd client-v2 && URL_PREFIX=${URL_PREFIX} trunk build --release

FROM rust:1 AS build-server

WORKDIR /src
COPY server server
RUN cd /src/server && cargo build --release

FROM debian:bookworm-slim AS app

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=build-server /src/server/target/release/simples /simples
COPY --from=build-client /src/client-v2/dist /dist

ENTRYPOINT ["/simples"]
