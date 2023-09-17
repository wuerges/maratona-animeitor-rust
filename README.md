# Maratona Animeitor

## Placar para live streaming do BOCA para uso no OBS

Este placar foi feito para as etapas regional e nacional da Maratona de Programação da SBC.

## Compilando e Rodando

Instale o docker, e:

```
docker compose up --build
```

Para visualizar as URLs cadastradas:

```
docker compose run printurls
```

# Linux

No linux, o animeitor vai criar uma conexa para cada cliente, por isso deve-se aumentar o numero de descritores:

```
ulimit -n unlimited
```

# Environment Variables

```
# Path to the file that contains the secrets used as credentials for the Reveleitor.
SECRET=/config/Regional_2023_Secrets.toml

# Path to the file that describes the contest locations.
SEDES=/config/Regional_2023.toml

# Boca URL that will be pooled to get the contest state.
# Can be either a file or an URL
BOCA_URL=/tests/inputs/webcast-2023-1a-fase-final-prova.zip

