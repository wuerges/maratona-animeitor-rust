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

