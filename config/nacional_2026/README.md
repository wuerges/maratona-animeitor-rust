# Nacional 2026 — 2ª fase

This directory uses the unified `event.toml` manifest and the multi-event feeder introduced by the refactor. It is
not compatible with the legacy `--sedes` and `--secret` server flags used by
`config/nacional_2025`.

Before running it, copy the root `secret_env.example` to `secret_env` and
`internal_tokens.toml.example` to `internal_tokens.toml` (both are not tracked),
or pass `BOCA_URL` to `make`. Public media and scoring settings belong in `.env`;
the root `secret_env` contains only private credentials.

The TOML contest/site mapping was copied from 2025 as a baseline and is selected
by `event.toml`. Verify every team-login prefix against the 2026 webcast before
production.

Salts are optional. The feeder only sends the salts explicitly configured in
`Secrets_secret.toml`; it never creates or rotates them. A site gets a stable,
random-looking reveal secret only when the event, contest, and site salts are
present. The secret is derived deterministically from those salts and the event,
contest, and site names, so restarting the server does not change it.

For a local rehearsal, generate the required certificate first. The root
`run-config` target then starts the server and feeder together, stopping the
server when the feeder exits:

```bash
make generate-dev-certs
make run-config CONFIG=config/nacional_2026/event.toml \
  BOCA_URL=tests/inputs/webcast_jones.zip
```

The certificate is valid for `localhost`, `127.0.0.1`, and `::1`.

For production, build and run the server and feeder separately:

```bash
make rebuild-server-for-release
make run-server
make run-feeder CONFIG=config/nacional_2026/event.toml BOCA_URL=tests/inputs/webcast_jones.zip
```

After the feeder has created the event, print its contest and reveal URLs with
`make printurls CONFIG=config/nacional_2026/event.toml`.
