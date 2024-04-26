# Maratona Animeitor

## Live Scoreboard to use with BOCA

This is the scoreboard used for South American ICPC contests.

## Running:

Install docker, and docker compose, then:

```
docker compose up --build
```

## URLs:

To see the urls served by Animeitor:

```
docker compose run printurls
```

# Basic configuration

Animeitor can be configured using a few environment variables:

```
# Path to the file that contains the secrets used as credentials for the Reveleitor.
SECRET=/config/Regional_2023_Secrets.toml

# Path to the file that describes the contest locations.
SEDES=/config/Regional_2023.toml

# Boca URL that will be pooled to get the contest state.
# It can be either a file or an URL.
BOCA_URL=/tests/inputs/webcast-2023-1a-fase-final-prova.zip

# Animeitor public hostname. This is set to `animeitor.naquadah.com.br` during the maratona.
# `localhost` is fine for local testing:
PUBLIC_HOST=localhost

# This is the public port. This is set to `80` during the SBC Maratona.
# `9000` is fine for local testing:
PUBLIC_PORT=9000

```

# Run without docker

The `Makefile` has an example of how to run animeitor without docker:

```
make run-standalone
```

## Important!

The public url is baked into the client.
The docker client image must be rebuilt every time `PUBLIC_HOST` or `PUBLIC_PORT` change.
