# Maratona Animeitor

## Live Scoreboard to use with BOCA

This is the scoreboard used for South American ICPC contests.

## Prerequisites:

- `docker` and `docker compose`.

## Running:

Install docker, and docker compose, clone the repo and bring the services up:

```
git clone https://github.com/wuerges/maratona-animeitor-rust
cd maratona-animeitor-rust
docker compose up
```

## URLs:

To see the urls served by Animeitor:

```
docker compose run printurls
```

# Basic configuration

Animeitor can be configured using a few environment variables, than can be set in the `.env` file:

```
# Path to the file that contains the secrets used as credentials for the Reveleitor.
# There are many examples in the ./config/ folder
SECRET=./config/basic_secret.toml

# Path to the file that describes the contest locations.
# There are many examples in the ./config/ folder
SEDES=./config/basic.toml

# Boca URL that will be pooled to get the contest state.
# It can be either a file or an URL.
BOCA_URL=./tests/inputs/webcast_jones.zip

# Animeitor API prefix. This is set to `http://animeitor.naquadah.com.br` during the maratona.
# `http://localhost:8000` is fine for local testing:
PREFIX=http://localhost:8000

# This is the public port. This is set to `80` during the SBC Maratona.
# `8000` is fine for local testing:
PUBLIC_PORT=8000
```

# Run without docker

The `Makefile` has an example of how to run animeitor without docker:

```
make run-standalone
```

To execute client with trunk:

```
make run-debug-client
```
