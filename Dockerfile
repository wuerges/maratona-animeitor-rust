FROM rust:1-bookworm AS build

WORKDIR /app
COPY . .
RUN cd server && cargo build --release

FROM debian:bookworm-slim AS app

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY ./client-v2/release /dist
COPY --from=build /app/server/target/release/simples /simples

ENTRYPOINT ["/simples"]
