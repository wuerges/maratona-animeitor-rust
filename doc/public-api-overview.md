# Consume an Animeitor event

The public API reads event scoreboards; configuration is performed through the authenticated internal API at `/internal/openapi.json` (`/internal/docs`). The public specification is at `/api/openapi.json` (`/api/docs`).

`GET /api/about` returns the server build without authentication or event setup:
`{"data":{"version":"2.4.0","commit":"0123456789ab"}}`.
The version comes from the workspace `Cargo.toml`; `commit` contains the first
12 hexadecimal characters of the build's commit hash, or `null` when unavailable.
Responses use `Cache-Control: no-store` so a deployment check reads current metadata.
Native builds detect Git at compile time. Source archives and Docker builds can
set `ANIMEITOR_GIT_COMMIT` to a full Git hash (Docker: `--build-arg`). The Makefile
and release workflow supply it automatically. No Git command runs per request.

Discover event identifiers with `GET /api/events`, then contest identifiers with `GET /api/events/{event_name}/contests`. Fetch the contest's `/contest` for selected teams, problems, and timing and `/config` for appearance and site choices. Identifiers are nonempty path segments; percent-encode them when constructing URLs. Public JSON responses contain `data` on success or `errors:[{code,message}]` on failure. Optional envelope fields are absent. There are no salts or derived keys in public configurations.

All time values are seconds since event start, not timestamps. Before the supplied event time reaches zero, contest state, config, and submissions return `403 not_started`. Event and contest discovery, the event timer, and remote control remain available. The landing page links upcoming contests to their countdown screens. The clock is supplied by a feeder or controller; the server does not advance it automatically.

Use WebSocket upgrades (`ws://` or `wss://`) for `/timer`, `/runs_ws`, and `/remote_control/{key}`. Successful upgrades return `101`; subsequent text frames carry bare JSON without envelopes. Their schemas are PublicTimer, Run, and ControlMessage respectively. Resource/pre-start handshake errors have no JSON body. A malformed WebSocket upgrade can also be rejected by the HTTP framework.

The timer sends its current state immediately and then changes, suppressing consecutive duplicates. The run stream replays history then sends new submissions and corrections. Replace prior values when an ID repeats. Runs at or after the event's freeze boundary have answer `?`, regardless of the actual answer. Filters are captured at connection time; reconnect and rebuild state after changing them. Freeze boundary changes apply to open connections and replay submissions with the current server-side masking. Clearing stored runs does not clear stream replay history.

`runs_secret` is the only authenticated public endpoint. Obtain a private revelation URL from an authorized operator or `GET /internal/events/{event_name}/revelation_urls`. Parse its `secret` query parameter and send it as `Authorization: Bearer <site-key>`; do not put the key in the API request URL. The key selects the site. It cannot bypass the pre-start restriction. After start, missing/invalid keys return `403 invalid_key`. Site regexes filter the event's runs; operators should keep them within their contest's selected teams. Salt rotation invalidates affected keys; request fresh links from the internal API.

Example public state response:

```json
{"data":{"event":"regional-2026","contest":"brasil","problems":["A","B"],"teams":[{"login":"teambr001","escola":"Example University","nome":"Example Team"}],"time_seconds":60,"score_freeze_time_seconds":14400,"penalty_seconds":1200}}
```

Example bare run message:

```json
{"id":1,"team_login":"teambr001","prob":"A","time_seconds":56,"answer":"Y"}
```

Remote-control channel keys are arbitrary shared identifiers, not site revelation keys. Valid messages are `{"y":120}`, `{"query":"sede=fiemg"}`, `"Hidden"`, and `{"Show":"teambr001"}`. The server forwards them to other connections on the same event/contest/channel; it does not echo them to the sender.



Resource limits: public reads use the last committed in-memory snapshot.
The server validates and loads storage at startup; writes publish only after commit.
A storage outage does not force public reads to reopen the database. One server
process owns a database; external edits and multiple writers are unsupported.

The public and internal APIs have separate request concurrency budgets (128 and
32 by default), a configurable processing deadline (15 seconds by default), and return 429 or 408 when those limits are
reached. Detached commit/publish tasks have an independent cap of 32, retained
until the task finishes even if the caller disconnects. JSON requests retain the framework's 2 MiB limit. WebSockets share a
configurable process budget (50,000 by default), no age limit while responsive, 4 KiB incoming frame/message
limits, and a 10-second write deadline. Reconnect with backoff. A lagged stream
is disconnected and must reconnect for a fresh replay; live buffers hold 1,024
messages.

Remote-control keys are secret server-issued capabilities, scoped to an event
and contest. An owner with a read-write internal token issues a session with
`POST /internal/events/{event_name}/contests/{contest_name}/remote_control` and
revokes it with `DELETE` at that path plus `/{key}`. Credentials expire after
seven days by default (configurable from one second to 30 days). Credentials
survive restart with SQLite; memory-backed servers remain transient. Unknown, expired, and revoked keys return 404 without allocating a room. There are at most 16 sessions per contest, 256
per process, and eight connections per session. Revocation disconnects existing
relays. Treat the key as a secret; do not put complete request URIs in proxy logs.
Remote-control incoming frames/messages are limited to 4 KiB, and writes to 10
seconds. Control traffic allows 30 incoming frames/second with a burst of 60; excess
traffic disconnects the sender. Query strings are at most 2,048 bytes and photo
team identifiers at most 256 bytes.


Public timer and runs streams send a WebSocket ping every 30 seconds. A matching
pong keeps the connection healthy; 90 seconds without a matching pong closes it.
Browsers answer ping frames automatically, including when no contest data changes.
Healthy public streams remain open across multi-hour contests without an hourly
replay/reconnect. Control-frame floods are limited to 30 frames/second (burst 60).
Remote-control credential expiration is independent of stream liveness. Remote
relays also use ping/pong heartbeats, so an idle healthy controller can remain
connected until its credential expires or is revoked. Deleting/recreating its
event or contest removes the credential.

Server budgets are configured in the optional `[limits]` table. Defaults are
admission ceilings; 20,000-viewer capacity still requires dedicated load testing.
