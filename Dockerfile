FROM rust:1.68.2-bullseye AS build


WORKDIR /src
COPY . .

RUN cargo install wasm-pack
RUN wasm-pack build --dev --target web --out-name package client
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release

FROM debian:buster-slim AS app

COPY --from=build /src/target/release/simples /simples

ARG HTTP_PORT
EXPOSE $HTTP_PORT

CMD ["/simples"]
