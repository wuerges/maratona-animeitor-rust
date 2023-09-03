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
wasm-pack build client --release --out-dir www/pkg --target web --out-name package
# rodando o servidor
cargo run --release --bin simples -- --config config/ICPC_LA.toml --secret config/Secret.toml ./tests/inputs/2a_fase_2021-22/brasil.zip
```

Mais opções podem ser examinadas com o comando help:

```bash
cargo run --release --bin simples -- --help
```

## Configurando o OBS e customizando o placar

A partir deste momento, o placar e os runs ficarão disponíveis nas URLs que o programa mostrar:

```bash
Maratona Rustreimator rodando!
-> Runs em http://localhost:8000/runspanel.html
-> Placar automatizado em http://localhost:8000/automatic.html
-> Timer em http://localhost:8000/timer.html
-> Painel geral em http://localhost:8000/everything.html
-> Fotos dos times em http://localhost:8000/teams.html
-> Painel geral com sedes em http://localhost:8000/everything2.html
-> Brasil
    Reveleitor em http://localhost:8000/reveleitor.html?secret=abcxyz&sede=Brasil
    Filters = ["teambrbr1"]
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
