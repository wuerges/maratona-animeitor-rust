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
python -mhttp.server --directory lib-server/test/
cargo make build_release
cargo run --release --bin simples http://0.0.0.0:8000/webcast_1573336220.zip

```

Os parâmetros necessários para rodar são a porta HTTP e a URL disponibilizada pelo BOCA.

O programa também suporta a leitura dos arquivos do webcast direto de um arquivo, se desejado:

```
cargo make build_release 
cargo run --release --bin simples lib-server/test/webcast_jones.zip
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

O placar e os runs podem ser customizados usando CSS, através do arquivo [static/styles.css](lib-server/static/styles.css). 





### Uso avançado

O rustreimeitor foi desenvolvido para apoiar a Maratona de Programação da SBC, 
e contém um arquivo de configuração específico para a maratona: `src/config.rs`.

Neste arquivo é possível declarar as sedes da prova.


### Uso acessando o database do boca

Se o Rustreimeitor está usando na mesma máquina do banco de dados do BOCA,
ele é capaz de acessar os dados do contest direto do banco (postgresql).

Para isso, é necessário:

1. Se você está no Ubuntu, instalar as libs do postgresql:

```
sudo apt-get install libpq-dev
```

2. Configurar a string de acesso do banco, que está no arquivo `.env`

O arquivo vem configurado com o banco, senha e usuários padrão do BOCA:

```
DATABASE_URL=postgres://bocauser:boca@localhost/bocadb
```

3. Descobrir os IDs do contest e do site do contest a mostrar o placar.

Na maioria dos casos, estes valores são 1 e 1.

4. Compilar e rodar o turbineitor:

O formato de execução é

```
cargo make build_release
cargo run -p turbineitor <porta http> <contest id> <site id>
```

Na maioria dos casos o comando executado é este abaixo:

```
cargo make build_release
cargo run -p turbineitor 3030 1 1
```
