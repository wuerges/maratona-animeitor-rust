#  Maratona Rustrimeitor
## Placar para live streaming do BOCA para uso no OBS

Este placar foi feito para as etapas regional e nacional da Maratona de Programação da SBC.

## Compilando e Rodando

Pré-requisitos:

- Se você está no Ubuntu 20.04, deve instalar o build-essential, as libs do openssl e o pkg-config:

```
sudo apt-get install build-essential libssl-dev pkg-config
```

- Instale o [Rust](https://www.rust-lang.org/pt-BR/tools/install)
- Instale o `wasm-pack`:

```
cargo install wasm-pack --version 0.9.1
# obs: neste momento o wasm-pack 0.10.x esta quebrado!
```

- Instale o `cargo-make`: 

```
cargo install cargo-make
```

Clone este repositório:

```
git clone https://github.com/wuerges/maratona-animeitor-rust
cd maratona-animeitor-rust
```

Compile e rode:

```
cargo make build_release
cargo run --release --bin simples <url_do_placar>
```

O repositório contém um exemplo, que pode ser usado para testes:

```
python -mhttp.server --directory server/test/
cargo make build_release
cargo run --release --bin simples http://0.0.0.0:8000/webcast_1573336220.zip

```

Os parâmetros necessários para rodar são a porta HTTP e a URL disponibilizada pelo BOCA.

O programa também suporta a leitura dos arquivos do webcast direto de um arquivo, se desejado:

```
cargo make build_release 
cargo run --release --bin simples server/test/webcast_jones.zip
```

Mais opções podem ser examinadas com o comando help:

```
cargo run --release --bin simples -- --help
```


## Configurando o OBS e customizando o placar

A partir deste momento, o placar e os runs ficarão disponíveis nas URLs que o programa mostrar:

```
Maratona Rustreimator rodando!
-> Runs em http://localhost:3030/seed/runspanel.html
-> Placar automatizado em http://localhost:3030/seed/automatic.html
-> Placar interativo em http://localhost:3030/seed/stepping.html
-> Timer em http://localhost:3030/seed/timer.html
-> Painel geral em http://localhost:3030/seed/everything.html
-> Reveleitor em http://localhost:3030/seed/reveleitor.html?secret=vYZgSm
```

Estas urls podem ser acessados no navegador, ou incluídas no OBS, através do browser incluso.

O placar e os runs podem ser customizados usando CSS, através do arquivo [client/static/styles.css](static/styles.css). 

