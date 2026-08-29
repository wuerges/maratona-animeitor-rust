# Maratona Rustrimeitor

## Placar para live streaming do BOCA para uso no OBS

Este placar foi feito para as etapas regional e nacional da Maratona de Programação da SBC.

## Compilando e Rodando

Pré-requisitos:

- Se você está no Ubuntu 20.04, deve instalar o build-essential, as libs do openssl e o pkg-config:

```bash
sudo apt-get install build-essential libssl-dev pkg-config
```

- Instale o [Rust](https://www.rust-lang.org/pt-BR/tools/install)
- Instale o `wasm-pack`:

```bash
cargo install wasm-pack
```

Clone este repositório:

```bash
git clone https://github.com/wuerges/maratona-animeitor-rust
cd maratona-animeitor-rust
```

Compile e rode:

```bash
# compilando o cliente
make rebuild-client-for-release
# rodando o servidor (a API interna usa o token; os volumes servem o cliente)
cargo run -p server-v2 --bin simples -- -t token-de-teste \
    -v ./client-v2/release: -v ./client-v2/release:animeitor
# em outro terminal, o feeder publica o estado do BOCA na API interna
cargo run -p cli --bin update_contest_state -- -t token-de-teste \
    -i ./tests/inputs/webcast_jones.zip -s http://localhost:8000
```

Mais opções podem ser examinadas com o comando help:

```bash
cargo run -p server-v2 --bin simples -- --help
```

## Configurando o OBS e customizando o placar

Os eventos, contests, sites e salts são criados pela API interna (`doc/event-api.md`);
o `printurls` lê a API interna e imprime as URLs do placar e do reveleitor:

```bash
cargo run -p cli --bin printurls -- --server http://localhost:8000 --token token-de-teste
-> brasil
    Animeitor em http://localhost:8000/animeitor/default/brasil/
    Reveleitor em http://localhost:8000/animeitor/default/brasil/?secret=abcxyz&sede=fiemg
```

# Desenvolvimento

```bash
# uma aba para monitorar o cliente
( cd client && cargo watch -x check )

# uma aba para rodar os testes
cargo watch -x test
```

# Usando Docker

Construindo a imagem:

```
docker compose up --build
```

# Linux

No linux, o animeitor vai criar uma conexa para cada cliente, por isso deve-se aumentar o numero de descritores:

```
ulimit -n unlimited
```

# Client only setup

The client can be redirected to another server, using an environment variable:

```bash
# generating the client pointing to animeitor
URL_PREFIX="http://animeitor.naquadah.com.br" wasm-pack build client --release --out-dir www/pkg --target web --out-name package

# serving the client assets locally
python3 -m http.server 8000 -d client/www
```
