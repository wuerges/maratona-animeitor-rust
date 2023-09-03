FROM rust:1.72.0-bookworm AS build-client

RUN cargo install wasm-pack

WORKDIR /src
COPY client client
COPY server/data server/data
COPY server/Cargo.toml server

ARG URL_PREFIX=http://localhost:8000
RUN URL_PREFIX=$URL_PREFIX wasm-pack build client --release --out-dir www/pkg --target web --out-name package

FROM nginx
COPY --from=build-client /src/client/www /usr/share/nginx/html
