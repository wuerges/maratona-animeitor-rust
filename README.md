# Maratona Animeitor

Live scoreboard for BOCA and South American ICPC contests.

## Run with Docker Compose

Install Docker with Compose, Make, and OpenSSL. From the repository root:

```bash
cp server.docker.toml.example server.toml
cp event-secrets.toml.example event-secrets.toml
make generate-dev-certs
make rebuild-docker-image
make run-docker
```

Rebuild the image after server, CLI, generator, or web-client code changes.
`make run-docker` regenerates `.generated/compose.json` from the TOML files and
starts the stack. After generation, plain `docker compose up` also works.
Stop it with `make stop-docker`. Docker does not require host Rust or Python.

- Public landing: http://localhost:8000/
- Scoreboards: `http://localhost:8000/animeitor/{event}/{contest}/`
- Internal docs: https://localhost:8443/internal/docs (named-token Basic auth).
- Optional Prometheus: http://localhost:9090/ — start with `make monitor-docker`.

Internal HTTP requests are rejected. Containers trust the configured CA without
disabling TLS verification. Browsers need to trust the local certificate for HTTPS
docs. Development certificate SANs include localhost, both loopback addresses,
and the Compose hostname animeitor.

`printurls` runs offline and exits after printing URLs; no running server or
webcast credentials are needed. Print them again with:

```bash
docker compose run --rm printurls
```

## Three configuration files

| File | Purpose | Committed? |
| --- | --- | --- |
| `config/<event>/event.toml` | One complete event, contests/sites, scoring/media settings, public derivation values | Yes |
| `event-secrets.toml` | Private webcast URL/path per event | No; copy the example |
| `server.toml` | Ports, TLS paths, API tokens, public/internal URLs, assets, private revelation salt | No; copy a Docker or host example |

The server reads **only server.toml**. It has no event-file paths or webcast
sources and receives event state through its internal API. Each feeder selects
one event and its corresponding entry from the minimal private mapping:

```toml
[webcasts]
nacional-2026 = "tests/inputs/webcast_jones.zip"
regional-2026 = "https://boca.example/webcast?key=private-credential"
```

Multiple feeder processes can publish different events to the same server.
Filesystem paths are relative to the file declaring them. Certificates and private
keys are separate ignored assets referenced by server.toml. Select files with:

```bash
make run-docker EVENT_CONFIG=config/regional_2026/event.toml \
  EVENT_SECRETS=event-secrets.toml SERVER_CONFIG=server.toml
```

Public event `secret` fields are derivation values, safe to commit. The event value
is required; contest/site values default to empty strings. Authentication also
requires the private server `revelation_salt`. Keep production server configuration
private and replace all development credentials. There is no legacy-key fallback.

Keys use HMAC-SHA256 with the private server salt and the compact JSON array
`["animeitor-site-key-v1", event_name, contest_name, site_name, event_secret, contest_secret, site_secret]`,
then the existing base62 encoding truncated to 12 characters. Offline printurls and
the server use the same function. These keys differ from the old salt-only keys.
Changing the server salt rotates all deployment keys; changing an event, contest,
or site value rotates that scope. Restart affected programs and redistribute URLs.

The Rust `animeitor-config compose` helper generates Compose and Prometheus
artifacts in the ignored `.generated` directory. Generated Compose contains
configuration paths, not tokens. The separate generated Prometheus token file is
private. Application containers receive only the configuration files they need;
the server never mounts event configuration files. Do not place those files under
server asset mounts. Docker mount sources are host paths; targets are container
paths. Production certificates/CA and assets can use configurable absolute paths.

The old `.env`, `secret_env`, standalone token TOML, and configuration CLI flags
are no longer read. Existing private files are not overwritten. Obsolete contest configurations and launchers have been removed; Git history
preserves the historical examples.

## Run without Docker

Install Rust, Trunk, Make, and OpenSSL. Use the host configuration example:

```bash
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
make run-feeder EVENT_CONFIG=config/nacional_2026/event.toml
make printurls EVENT_CONFIG=config/nacional_2026/event.toml
```

The binary interfaces are:

```text
animeitor-server --server-config server.toml
animeitor-feeder --event-config config/nacional_2026/event.toml --event-secrets event-secrets.toml --server-config server.toml
printurls --event-config config/nacional_2026/event.toml --server-config server.toml
```

For a minimal event, use `make run-basic`. The host example uses localhost for the
internal URL and the release client directory for assets; the Docker example uses
animeitor and /dist. Both use ports 8000 and 8443. To develop the client interactively,
run `make run-debug-client` alongside the server and open http://localhost:8080/.

## Appearance and file descriptors

Edit `animeitor-client/static/user-styles.css` and restart the Docker server to
reload assets. Outside Docker, rebuild the release client first. Refresh the browser
afterward. Configure photos/sounds and other static mappings in server.toml.

Compose and systemd examples raise the nofile limit to 65536 because each websocket
uses a descriptor. For local shell deployments, use `ulimit -n 65536` as needed.

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
