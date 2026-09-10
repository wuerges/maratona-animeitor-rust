# Nacional 2026 — 2ª fase

This directory uses the multi-event feeder introduced by the refactor. It is
not compatible with the legacy `--sedes` and `--secret` server flags used by
`config/nacional_2025`.

Before running it, set the 2026 webcast source in `secret_env` (which is not
tracked) or pass `BOCA_URL` to `make`. `secret_env` may also set
`INTERNAL_TOKEN`, `PHOTO_URL_FORMAT`, `SOUND_URL_FORMAT`,
`SCORE_FREEZE_TIME_SECONDS`, and `SENTRY_DSN`.

The TOML contest/site mapping was copied from 2025 as a baseline. Verify every
team-login prefix against the 2026 webcast and decide whether the three
reference-only CCL configs belong in `CONTEST_FILES` before production.

For a local rehearsal, `run-standalone-loop` starts the server and the feeder
together, stopping the server when the feeder exits:

```bash
make -f config/nacional_2026/Makefile run-standalone-loop \
  BOCA_URL=/path/to/webcast.zip
```

For production, build and run the server and feeder separately:

```bash
make -f config/nacional_2026/Makefile rebuild-release-server
make -f config/nacional_2026/Makefile run-standalone
make -f config/nacional_2026/Makefile run-feeder BOCA_URL=https://example/webcast.zip
```

After the feeder has created the event, print its contest and reveal URLs with
`make -f config/nacional_2026/Makefile printurls-standalone`.
