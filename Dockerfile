FROM rust:1.72.0-bookworm AS build-base

RUN cargo install wasm-pack
RUN cargo install cargo-chef

WORKDIR /src

FROM build-base AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM build-base AS build
COPY --from=planner /src/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN wasm-pack build client --release --out-dir www/pkg --target web --out-name package
RUN cargo build --release

FROM debian:bookworm-slim AS app
COPY --from=build /src/target/release/simples /simples
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

ARG HTTP_PORT
EXPOSE $HTTP_PORT

ENTRYPOINT ["/simples"]
