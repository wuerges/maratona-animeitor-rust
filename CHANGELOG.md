# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Split the `data` crate into pure dump/wire types: client-side domain logic
  (scoring engine, reveal driver, panel views, client helpers) moved to
  `client-model`; server-only dump items (contest state, secret config,
  server-side run transforms) moved to `service`. No wire-format changes.
- Shrunk `server-v2` to the minimal actix layer: state, endpoint data access,
  relay types and rustls loading moved to `service`; the `simples` server
  binary moved from `cli` into `server-v2`; `cli` keeps only the tools and
  took over the CLI-only helpers (`pair_arg`, `sentry`, secret-file config).
- Contests now require a non-empty name: the previous default contest `""`
  (empty URL segment) no longer exists, and empty names are rejected with
  `invalid_value`. The BOCA feeder creates the contest `default`.

### Added

- `GET /api/events/{event}/contests` lists the contest names of an event
  (alphabetical; `403 not_started` before the start) so the landing page can
  link each contest.
- One server now hosts many events, created and fed through a new private
  internal API (`/internal`, HTTP Basic token): event/contest/site CRUD,
  runs, countdown times, and per-level salts whose derived site keys unlock
  the reveal. All times are in seconds, and the unit is part of the field
  name.
- The public API was reorganized under `/api/events/{event}/contests/{contest}/...`,
  mirroring the internal hierarchy. Before a contest starts, its endpoints
  answer `403` (`not_started`) — only the event list and the event timer stay
  available, so the countdown screen reveals nothing about the contest.
- The client is served at `/animeitor/{event}/{contest}/`: a single build
  discovers the event and contest from the URL path, a landing page at `/`
  lists the active events, and a countdown screen is shown before the start.
- Reveal runs are unlocked by a site key sent in the `Authorization: Bearer`
  header instead of a `?secret=` query parameter, and photo/sound URLs come
  from the contest configuration instead of the deploy config.
- BOCA feeding moved out of the server into a standalone feeder process that
  publishes through the internal API (also creates the contest `default`).
- `printurls` was rewritten to read events, contests, sites and salts from the
  internal API and to print the contest and reveleitor URLs (with the derived
  site keys).

### Removed

- The legacy public endpoints (`/api/contest`, `/api/config`,
  `/api/allruns_ws`, `/api/allruns_secret`, `/api/timer`,
  `/api/remote_control/{key}` and `PUT /api/contests`) and the old
  `--sedes`/`--secret`/`--salt`/`-i` CLI flags: events are configured
  exclusively through the internal API.
