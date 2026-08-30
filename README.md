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

The client is served per event and contest at `/animeitor/{event}/{contest}/`,
and `http://localhost:8000/` lists the active events. The compose setup feeds
BOCA into the `default` event, so with the defaults:

- Animeitor: http://localhost:8000/animeitor/default/
- Reveleitor: the URL printed by `printurls` (e.g.
  `http://localhost:8000/animeitor/default/?secret=<site-key>&sede=<site>`)

Events, contests, sites and salts are created through the internal API
(`doc/event-api.md`) — the feeder creates the `default` event and contest
automatically.

# Basic configuration

Animeitor can be configured using a few environment variables, than can be set in the `.env` file:

```bash
# Boca URL that will be polled by the feeder to get the contest state.
# It can be either a file or an URL.
BOCA_URL=./tests/inputs/webcast_jones.zip

# Animeitor API prefix used to print the contest/reveleitor URLs.
# This is set to `http://animeitor.naquadah.com.br` during the maratona.
# `http://localhost:8000` is fine for local testing:
PREFIX=http://localhost:8000

# URL of the animeitor server, used by printurls and the feeder.
SERVER_URL=http://animeitor:8000

# This is the public port. This is set to `80` during the SBC Maratona.
# `8000` is fine for local testing:
PUBLIC_PORT=8000

# Token for the internal API (/internal). The feeder and printurls need the same token.
INTERNAL_TOKEN=token-de-teste
```

# Customizing animeitor appearance

There is a special CSS file at `client-v2/static/user-styles.css`.
This file is included in the build and mounted by docker.
It can be edited and overwrites the CSS from animeitor. The client assets
(including this file) are loaded into memory once when the server starts,
so after editing, restart the server (`docker compose restart animeitor`)
and reload the browser with `ctrl+shift+R` to see the changes.

```css
/* This file is intended to house user CSS */
/* It will not be included in the minimizer, but it will be used in the app */

/* remove this comment to make the background of animeitor yellowgreen
body {
  background-color: yellowgreen;
}
*/
```

Animeitor was made to be customizable using CSS.

# File descriptors

Each websocket connection holds a file descriptor for its lifetime, so a
production server needs a raised `nofile` limit (the default soft limit is
1024, which runs out quickly):

- systemd service (`config/regional_2026/simples.service`): `LimitNOFILE=65536`
  in the `[Service]` section
- docker compose: `ulimits: nofile: 65536` (already set in the compose files)
- running from a shell: `ulimit -n 65536` before starting

Client assets are served from memory, so they cost no file descriptors.

# Run without docker

The `Makefile` has an example of how to run animeitor without docker.

## Running local server using the prebuilt release client

```
make rebuild-client-for-release
make run-standalone-push
```

Then check your browser:

- Landing: http://localhost:8000/
- Animeitor: http://localhost:8000/animeitor/{event}/{contest}/

To also feed BOCA while running without docker, use `make run-standalone-loop`
(it starts the server and the feeder together).

## Running the debug client

In other terminal, without closing the server above:

```
make run-debug-client
```

Then check your browser:

- Landing: http://localhost:8080/
- Animeitor: http://localhost:8080/animeitor/{event}/{contest}/


## Dependencies

- `rust`: https://rustup.rs/
- `trunk`: To install `trunk`, visit the project page: https://trunk-rs.github.io/trunk/

All project dependencies have been updated in september 16, 2025.

## Rebuilding the docker image:

```
make rebuild-docker-image
```

# Keyboard shortcuts:

| Key | Name        | Function                              |
| --- | :---------- | :------------------------------------ |
| `y` |             | Open/close team photo                 |
| `m` |             | Enable/disable autoplay for team song |
| `⌫` | Backspace   | Reset revelation                      |
| `←` | Arrow left  | Step back one submission              |
| `→` | Arrow right | Step forward one submission           |
| `↑` | Arrow up    | Step up one team                      |
| `↓` | Arrow down  | Step down one team                    |
