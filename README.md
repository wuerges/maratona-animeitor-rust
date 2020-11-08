#  Maratona Rustrimeitor
## Placar para live streaming do BOCA para uso no OBS

Este placar foi feito para as etapas regional e nacional da Maratona de Programação da SBC.

## Compilando e Rodando

Pré-requisitos:

- Se você está no Ubuntu 20.04, deve instalar as libs do openssl:

```
sudo apt-get install libssl-dev
```

- Instale o [Rust](https://www.rust-lang.org/pt-BR/tools/install)
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
cargo run --release --bin simples -p lib-server <porta_tcp> <url_do_placar>
```

O repositório contém um exemplo, que pode ser usado para testes:

```
python -mhttp.server --directory lib-server/test/
cargo make build_release
cargo run --release --bin simples -p lib-server 3030 http://0.0.0.0:8000/webcast_1573336220.zip

```

Os parâmetros necessários para rodar são a porta HTTP e a URL disponibilizada pelo BOCA.

O programa também suporta a leitura dos arquivos do webcast direto de um arquivo, se desejado:

```
cargo make build_release 
cargo run --release --bin simples -p lib-server 3030 lib-server/test/webcast_jones.zip
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

O placar e os runs podem ser customizados usando CSS, através do arquivo [static/styles.css](lib-server/static/styles.css). 





### Uso avançado

O rustreimeitor foi desenvolvido para apoiar a Maratona de Programação da SBC, 
e contém um arquivo de configuração específico para a maratona: `src/config.rs`.

Neste arquivo é possível declarar as sedes da prova.

Para rodar o rustreimeitor no modo maratona, execute ele desta maneira:

```
cargo make build_release 
cargo run --release --bin maratona -p lib-server 3030 <Url do Boca> <Sal Secreto do Boca>
```