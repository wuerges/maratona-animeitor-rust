FROM debian:bookworm-slim AS app

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY ./client-v2/release /dist
COPY ./server/target/x86_64-unknown-linux-musl/release/simples /simples
COPY ./server/target/x86_64-unknown-linux-musl/release/printurls /printurls
COPY ./config /config
COPY ./server/photos/fake.webp /photos/fake.webp
COPY ./server/sounds/applause.mp3 /sounds/applause.mp3
COPY ./tests /tests

ENTRYPOINT ["/simples"]
