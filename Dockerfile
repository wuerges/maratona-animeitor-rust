FROM rust:1.68.2-bullseye

ARG HTTP_PORT

WORKDIR /src
COPY . .

RUN cargo install wasm-pack
RUN wasm-pack build --dev --target web --out-name package client
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release

EXPOSE $HTTP_PORT

ENTRYPOINT ["cargo", "run", "--release", "--bin", "simples", "--"] 
