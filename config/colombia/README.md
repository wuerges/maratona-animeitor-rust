# Colombia example

One event, `colombia`, with two contests:

- `Teams`: any login containing `team`.
- `CCLs`: any login containing `ccl`.

Each contest has one `General` site containing all its teams. Login filters are
case-sensitive substring matches. A login containing both strings appears in
both contests; logins containing neither are excluded.

Set up `server.toml` and `event-secrets.toml` as described in the
[main README](../../README.md). Add the event's actual private webcast URL or
local ZIP path to the `[webcasts]` table in `event-secrets.toml`:

```toml
[webcasts]
colombia = "https://example.com/private/webcast.zip"
```

The webcast supplies the teams and runs for both contests. From the repository
root, run:

```bash
make run-config EVENT_CONFIG=config/colombia/event.toml
```

To print scoreboard and revelation URLs offline:

```bash
make printurls EVENT_CONFIG=config/colombia/event.toml
```

## Docker

From the repository root, generate the local development certificate:

```bash
make generate-dev-certs
```

Create or edit the root `event-secrets.toml` and add the `colombia` entry to its
`[webcasts]` table using the actual URL shown above. For a local ZIP, put it in
`config/colombia/webcasts/` and use the container path instead:

```toml
[webcasts]
colombia = "/workspace/webcasts/colombia.zip"
```

Start the Colombia stack using the same published image as every other event:

```bash
cd config/colombia
docker compose up
```

Open http://localhost:8000/animeitor/. The `printurls` service also prints direct
links for both contests and their revelation pages. Event settings and webcast
sources are mounted at runtime; no event-specific image is needed. Run
`docker compose pull` to fetch a newer published image.

To include local source changes that have not been published, build the shared
image from the repository root, then start Compose without pulling:

```bash
docker build -t wuerges/animeitor:latest .
cd config/colombia
docker compose up --pull never
```

This local setup uses `server.docker.toml.example` and its development credentials.
It stores event state in memory and the feeder resets the selected event on
startup. It uses the same ports as the root Jones demo, so stop that stack first
if it is running. Run `docker compose down` from this directory to stop Colombia.
