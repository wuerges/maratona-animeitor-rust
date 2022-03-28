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
cargo install wasm-pack
```

Clone este repositório:

```
git clone https://github.com/wuerges/maratona-animeitor-rust
cd maratona-animeitor-rust
```

Compile e rode:

```
( cd client && wasm-pack build --target web --out-name package )
cargo run --release --bin simples <url_do_placar> --port 9091
```

O programa também suporta a leitura dos arquivos do webcast direto de um arquivo, se desejado:

```
cargo make build_release
cargo run --release --bin simples ./tests/inputs/webcast_jones.zip
```

Mais opções podem ser examinadas com o comando help:

```
cargo run --release --bin simples -- --help
```


## Configurando o OBS e customizando o placar

A partir deste momento, o placar e os runs ficarão disponíveis nas URLs que o programa mostrar:

```
Maratona Rustreimator rodando!
-> Runs em http://localhost:9091/runspanel.html
-> Placar automatizado em http://localhost:9091/automatic.html
-> Timer em http://localhost:9091/timer.html
-> Painel geral em http://localhost:9091/everything.html
-> Fotos dos times em http://localhost:9091/teams.html
-> Painel geral com sedes em http://localhost:9091/everything2.html
-> Reveleitor em http://localhost:9091/reveleitor.html?secret=abc
```

Estas urls podem ser acessados no navegador, ou incluídas no OBS, através do browser incluso.

O placar e os runs podem ser customizados usando CSS, através do arquivo [client/static/styles.css](client/static/styles.css).
