FROM rust:1.67.1

WORKDIR /src
COPY . .

RUN cargo install wasm-pack
RUN wasm-pack build --dev --target web --out-name package client
RUN cargo build --release

EXPOSE 9091

ENTRYPOINT ["cargo", "run", "--release", "--bin", "simples", "--", "--port", "9091"]
