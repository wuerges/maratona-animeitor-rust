# Maratona Animeitor

Animeitor displays an animated scoreboard for programming contests using BOCA
webcasts. Its Reveleitor mode lets you reveal frozen results step by step during
the awards ceremony.

## Try the example scoreboard

Download or clone this repository and open a terminal in its root directory.
You need Docker with Docker Compose, Make, and OpenSSL installed, with Docker
running. Then run:

```bash
make generate-dev-certs
docker compose up
```

Open **http://localhost:8000/animeitor/jones/Jones/** in your browser. Allow a few
seconds for the scoreboard to load. The example contains 20 teams, 8 problems,
and 134 submissions from the Jones contest. It displays a saved webcast snapshot;
it is not connected to a live contest.

Docker downloads the published Animeitor image automatically. You do not need
Rust or a local build. The first command creates a local development certificate
so the services can communicate securely; the scoreboard itself opens over HTTP.

The terminal also prints Reveleitor links for revealing the results. To print
them again, open another terminal in the same directory and run:

```bash
docker compose run --rm printurls
```

Stop the example with `docker compose down`. To update an existing installation
to a newly published image, run `docker compose pull` followed by
`docker compose up -d`.

## Customize the contest and scoreboard sections

Start with [config/jones/event.toml](config/jones/event.toml), which is already
mounted by the example. A **contest** is a scoreboard with its own title and URL.
A **site** is a subsection of that scoreboard, such as a campus or team category;
it appears in the scoreboard navigation and has its own Reveleitor link.

Replace the file's contents with this example to rename the scoreboard and add
sections for the Jones teams whose logins begin with `arq` and `ctd`:

```toml
[event]
name = "jones"
secret = "development-event-value"

[[contests]]
name = "My Contest"
codes = [""]

[[contests.sites]]
name = "Everyone"
codes = [""]

[[contests.sites]]
name = "ARQ"
codes = ["^arq"]

[[contests.sites]]
name = "CTD"
codes = ["^ctd"]
```

`codes` contains regular expressions matched against **team logins**, not display
names. A team matches if any expression matches; `[""]` includes everyone.
Use prefixes that actually exist in your webcast, otherwise the section will be
empty. Site filters should select teams included by their parent contest.

Apply your changes and print the new links:

```bash
docker compose up -d --force-recreate
docker compose run --rm printurls
```

The renamed scoreboard is at
http://localhost:8000/animeitor/jones/My%20Contest/.
Select ARQ or CTD in its navigation to see that section. Renaming a contest also
changes its URL and revelation keys, so use the newly printed links.

To create another scoreboard from the same webcast, append another
`[[contests]]` block with a unique name and its own filters and sites. Keep
`event.name = "jones"` while trying the supplied fixture. For your own live BOCA
contest, select a matching event configuration and private webcast source as
explained under [Configuration files](#configuration-files).

## Customize the scoreboard styles

Edit [animeitor-client/static/user-styles.css](animeitor-client/static/user-styles.css)
to change the appearance. The file includes examples for changing problem balloon
colors. For example, add this to change the font:

```css
body {
  font-family: "Trebuchet MS", sans-serif;
}
```

The example Compose setup mounts this CSS file directly. Run
`docker compose restart animeitor`, then refresh your browser to see the change.
CSS-only changes do not require rebuilding the Docker image.

This repository is **vibe-code friendly**: Codex, Claude, or another coding
assistant can help customize colors, typography, layouts, and animations. Give
it a concrete description or a reference image, and ask it to inspect the
existing components and styles. For example:

> Customize the scoreboard with a dark navy theme and larger team names for a
> projector. Start with animeitor-client/static/user-styles.css, preserve score
> readability and revelation controls, and explain how to preview the changes.

For changes to the Rust client components or other application code, rebuild and
restart:

```bash
make rebuild-docker-image
docker compose up -d --force-recreate
```

Team photos and songs can also be configured through event/contest media URL
formats and server asset mounts, described below.

## Reveleitor hotkeys

Open a Reveleitor link printed by `printurls`. A welcome dialog explains that
the scores are not final until all submissions have been revealed and lists the
keyboard controls. Click **OK** to open Reveleitor, then click the page if it needs
keyboard focus. These controls let you present the frozen results interactively:

| Key | Action |
| --- | --- |
| `→` | Step forward one submission |
| `←` | Step back one submission |
| `↑` | Step up one team |
| `↓` | Step down one team |
| `Backspace` | Reset the revelation |
| `y` | Open or close the team photo |
| `m` | Enable or disable automatic playback of the team song |

Photos and songs require configured media. Keep real contest revelation links
private until you intend to reveal the results: they authorize access to frozen
submissions for the selected site.

## Architecture

The application has three programs and a browser client:

| Component | Responsibility | Configuration it reads |
| --- | --- | --- |
| `animeitor-server` | Stores event state in memory, serves the scoreboard and APIs, streams updates over WebSockets | Server only |
| `animeitor-feeder` | Reads a BOCA webcast URL or ZIP file and publishes teams, problems, runs, and contest/site definitions | Event, event secrets, server |
| `printurls` | Prints public scoreboard and private revelation links offline | Event and server |
| Browser client | Renders the animated scoreboard and Reveleitor | Public server API and the selected URL |

The feeder sends updates to the server over authenticated HTTPS. The server has
no knowledge of event configuration files or webcast credentials; it learns events
through the internal API. The browser reads the public API over HTTP and receives
live updates over WebSockets. Restarting the server clears its in-memory state;
the feeder repopulates it from the webcast.

The checked-in [docker-compose.yaml](docker-compose.yaml) starts one server, one
Jones feeder, and the one-shot `printurls` program. It is not generated. Multiple
feeders can publish different events to the same server.

### APIs and monitoring

- Public landing: http://localhost:8000/
- Internal API docs: https://localhost:8443/internal/docs
- Prometheus: http://localhost:9090/ after `docker compose --profile monitoring up -d`

The development internal API uses Basic authentication with username `feeder`
and password `development-token`. Internal requests over HTTP are rejected.
The feeder and Prometheus verify HTTPS using the mounted CA certificate. Your
browser needs to trust the development certificate to open the HTTPS docs without
a certificate warning.

[prometheus/prometheus.yml](prometheus/prometheus.yml) contains the development
HTTPS target, named token, and CA path. For production, mount a private monitoring
configuration with your credentials and CA. Compose raises the server's open-file
limit to 65536 for WebSocket connections; for host deployments, use
`ulimit -n 65536` as needed.

## Configuration files

All three configuration files use TOML. The demo mounts committed development
examples directly, so you can try it without copying private files.

| File | Contents | Version control |
| --- | --- | --- |
| `config/<event>/event.toml` | Event and contest/site definitions, public derivation values | Commit it |
| `event-secrets.toml` | Webcast URL or local path per event | Ignored; example provided |
| `server.toml` | Ports, TLS files, API tokens, URLs, assets, private revelation salt | Ignored; examples provided |

### Event configuration

The complete minimal example is shown in the customization section. See
[config/jones/event.toml](config/jones/event.toml) for the default and
[config/nacional_2026/event.toml](config/nacional_2026/event.toml) for a larger event.

| Table | Fields |
| --- | --- |
| `[event]` | Required `name` and `secret`; optional `score_freeze_time_seconds`, `photo_url_format`, `sound_url_format` |
| `[[contests]]` | Required unique `name` and `codes`; optional `secret`, `style`, `ouro`, `prata`, `bronze`, and media URL formats |
| `[[contests.sites]]` | Required `name` unique within its contest and `codes`; optional `secret` |

Each site belongs to the most recent `[[contests]]` block. Contest/site secrets
default to empty strings. Medal settings default to `ouro = 1`, `prata = 2`,
and `bronze = 3`. Without a freeze override, the webcast supplies the freeze time.
Contest media formats override event defaults. The contest `style` field names
a CSS class for its heading; define that class in `user-styles.css`.

Despite their name, event/contest/site `secret` fields are public key-derivation
values and can be committed. Revelation access also requires a key derived with
the private server salt; the public values alone do not grant access.

### Private webcast sources

Copy [event-secrets.toml.example](event-secrets.toml.example) to
`event-secrets.toml` and set the source for your event:

```toml
[webcasts]
my-event = "https://boca.example/webcast?key=private-credential"
```

The mapping key must equal `event.name`. A local ZIP path, such as
`tests/inputs/webcast_jones.zip`, is also accepted. Local paths are relative to
this configuration file. Each feeder selects one entry; adding entries does not
start additional feeders. The Jones fixture is intended for the Jones example;
real event manifests need their own matching webcast.

### Server configuration

Use [server.docker.toml.example](server.docker.toml.example) for Docker or
[server.toml.example](server.toml.example) for host execution. Copy the appropriate
example to `server.toml` before configuring a real deployment.

| Fields | Meaning |
| --- | --- |
| `public_port`, `tls_port` | Distinct HTTP and HTTPS listening ports; development defaults are 8000 and 8443 |
| `tls_cert`, `tls_key`, `tls_ca_cert` | Server certificate/key and the CA trusted by the feeder |
| `server_url` | HTTPS address used by the feeder; `https://animeitor:8443` inside Compose |
| `public_url` | Browser-facing base address used by `printurls` and API revelation URLs |
| `client_token` | Name of an enabled entry in `[[tokens]]` to use for API requests |
| `[[tokens]]` | Named credentials with `name`, `token`, and optional `enabled` (defaults to true) |
| `revelation_salt` | Private deployment-wide salt used to derive revelation keys |
| `[[assets]]` | Static file mappings, each with a filesystem `directory` and URL `path` |

Replace development tokens and the revelation salt for production, and configure
your certificate, key, and CA. These private TOML files and certificate/key files
are ignored by Git.

Keys use HMAC-SHA256 with the private server salt and the compact JSON array
`["animeitor-site-key-v1", event_name, contest_name, site_name, event_secret, contest_secret, site_secret]`,
then base62 encoding truncated to 12 characters. The server and offline
`printurls` use the same function. Changing the server salt rotates every key;
changing a name or derivation value rotates keys for the affected scope. Restart
affected programs and distribute newly printed URLs. There is no legacy-key fallback.

### Mount your own configuration in Compose

Change the Compose bind sources from `server.docker.toml.example` to
`server.toml` for all three programs, and from `event-secrets.toml.example` to
`event-secrets.toml` for the feeder. For a different event file, change its bind
source for both feeder and printurls. You may keep the existing container target
path; if you change it, update the corresponding `--event-config` arguments too.

Paths inside TOML resolve relative to the configuration file **inside the
container**. Mount local webcasts, certificates, keys, CAs, and assets at the
configured paths. Keep server mounts limited to server configuration, TLS files,
and public assets. If ports change, update the Compose port mappings, TOML
ports/URLs, and monitoring target together. Mount definitions belong in Compose;
old `[docker.mounts]` entries must be removed from server TOML.

Programs load configuration at startup. Apply changes with
`docker compose up -d --force-recreate`. Existing private files are not used by
the demo until you change its bind mounts.

The command-line interfaces are:

```text
animeitor-server --server-config server.toml
animeitor-feeder --event-config config/jones/event.toml --event-secrets event-secrets.toml --server-config server.toml
printurls --event-config config/jones/event.toml --server-config server.toml
```

## Run without Docker

Install Rust through rustup, Make, a C build toolchain, Perl, pkg-config, and the
OpenSSL development libraries. On Debian/Ubuntu, the native dependencies are:

```bash
sudo apt-get install build-essential perl pkg-config libssl-dev openssl
```

Install the client toolchain and Trunk, then use the host configuration example:

```bash
rustup toolchain install nightly --profile minimal --target wasm32-unknown-unknown
cargo install --locked trunk
cp server.toml.example server.toml
cp event-secrets.toml.example event-secrets.toml
make generate-dev-certs
# Match the wasm-bindgen CLI to Cargo.lock before building the web client.
cargo install --locked "wasm-bindgen-cli@$(awk -F'"' '/^name = "wasm-bindgen"$/{f=1} f && /^version = /{print $2; exit}' Cargo.lock)"
make rebuild-client-for-release
make run-config
```

Or start the programs independently:

```bash
make run-server
make run-feeder EVENT_CONFIG=config/jones/event.toml
make printurls EVENT_CONFIG=config/jones/event.toml
```

`make run-basic` also runs the Jones example. The host example uses localhost for the
internal URL and the release client directory for assets; the Docker example uses
animeitor and /dist. Both use ports 8000 and 8443. To develop the client interactively,
run `make run-debug-client` alongside the server and open http://localhost:8080/.
