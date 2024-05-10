FROM debian:bookworm-slim AS app

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY ./client-v2/release /dist
COPY ./server/target/x86_64-unknown-linux-musl/release/simples /simples

ENTRYPOINT ["/simples"]
