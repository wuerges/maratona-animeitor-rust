# Animeitor server

See the [root README](../README.md) for the current Docker and host workflows.

The server is independent of event configuration:

```bash
cargo run -p server-v2 --bin animeitor-server -- --server-config server.toml
```

Run commands from the repository root. Copy a server configuration example and
generate the development certificates first. The feeder reads the public event
file and the private event-to-webcast mapping, then publishes state over HTTPS:

```bash
cargo run -p cli --bin animeitor-feeder -- \
  --server-config server.toml --event-config config/nacional_2026/event.toml \
  --event-secrets event-secrets.toml
cargo run -p cli --bin printurls -- \
  --server-config server.toml --event-config config/nacional_2026/event.toml
```

`printurls` works offline. Both it and server authentication derive keys using the
private server salt and the public event/contest/site values. Old keys without the
server salt are not accepted.

Historical Compose files for older images have been removed. Use the root
Makefile and checked-in Compose configuration for current deployments.

## API documentation and event setup

The authenticated `/internal/docs` Swagger page and `/internal/openapi.json`
contain the complete [event setup walkthrough](../doc/internal-api-setup.md),
request examples, schemas, and error handling. Use the configured HTTPS endpoint
with HTTP Basic authentication (enabled username and its token). Public API docs
are at `/api/docs` and `/api/openapi.json`.

To obtain current revelation links for an API-configured event, request
`GET /internal/events/{event_name}/revelation_urls` with the same credentials.
It returns `data: [{contest, site, url}]`, sorted by contest and site, using the
configured `public_url` origin. Each private URL contains the site's `secret`
Bearer key and `sede` selection; no separate key endpoint is needed. Retrieve new
URLs after salt rotation. Unlike offline `printurls`, this endpoint uses the
server's current in-memory event configuration.
