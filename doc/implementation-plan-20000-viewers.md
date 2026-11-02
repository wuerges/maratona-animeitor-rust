# Implementation plan: 20,000 browser viewers on direct Axum

Status: implementation started; capacity validation is pending.
Implementation base: `3bfdadb2` (main after merging the reviewed candidate).
Review baseline: `v2.4.0`. Reviewed candidate: `3e66865e` (workspace 5.0.0).

## Instructions for the implementing agent

Execute the numbered work packages in order. Read `AGENTS.md` first. Inspect the
current tree before editing: subsequent work may already implement part of this
plan. Preserve unrelated changes. Complete and test each package before moving
on; record progress and evidence in the checklist at the end. Do not merely raise
the connection limit and declare capacity achieved.

Before implementation, record `git rev-parse HEAD` as `IMPLEMENTATION_BASE` in
the progress notes and shell environment. Keep that original SHA throughout the
work, even if creating intermediate commits. Do not recapture it at the end.

The user requested this plan after a code review. This document is not itself
authorization to deploy, publish a release, or run disruptive load against
production. Implementation can be done locally when this plan is assigned.
Large capacity tests require a dedicated target, not the production service.

### Fixed requirements

- Support at least **20,000 simultaneous browser viewers** on one server process.
  Each ordinary viewer opens a timer and a runs WebSocket: **40,000 sockets**,
  plus ordinary HTTP traffic and operational headroom.
- Production exposes Axum directly. Do not require nginx, a CDN, or a new proxy.
- All WebSocket clients are browsers. Keep server ping/pong heartbeats; browsers
  answer automatically. Do not add application-level heartbeat messages or
  browser JavaScript pong handling.
- Preserve public JSON envelopes, bare run/timer WebSocket messages, freeze and
  revelation semantics, token roles, event ownership, and existing URLs.
- Keep two public sockets per viewer in this project. Multiplexing, a new wire
  protocol, multi-process writers, and distributed storage are out of scope.
- Keep durable remote credentials, their limits, expiry, revocation, hashing,
  and log redaction. Remote control is not required for the 20,000-viewer test,
  but it must remain functional.
- Existing server configuration files must load with omitted new settings.
- Do not weaken security boundaries, make buffers unbounded, or silently discard
  submission corrections to meet a benchmark.

### Evidence already obtained

Both documented database/API test suites passed at the reviewed commit.
`make api-check API_BASE=v2.4.0` passed using `tufin/oasdiff:v1.30.0`; it classified
the schema changes as non-breaking. This does not check behavioral compatibility.
Three temporary reproductions confirmed:

1. An initialized store serves public cached reads during a database outage, but
   `get_event()` then calls `refresh()`, removes the event on a read failure, and
   invalidates initialization. Public reads subsequently fail and streams close.
2. Sending 1,025 new runs in one store operation can produce `Lagged(1)` on an
   existing runs receiver. The HTTP handler closes the socket on any receive error.
3. A paused timer write returns HTTP 408 after 15 seconds, then commits when the
   database is released. The detached task survives the timeout.

Recreate permanent regression tests from these descriptions; do not depend on
temporary files from the review session.

## 1. Centralize limits and remove the 1,024-viewer ceiling

Files: `server/service/src/app_config.rs`, new
`server/service/src/resource_limits.rs`, `server/service/src/lib.rs`,
`server/cli/src/configuration.rs`, `server/server-v2/src/main.rs`,
`server/server-v2/src/lib.rs`, `server/service/src/event_store.rs`,
`server/server-v2/src/resource_limits.rs`, server configuration examples.

Implementation:

1. Introduce one validated, serializable configuration type for server limits,
   exposed through an optional `[limits]` table with serde defaults and unknown
   field rejection. Pass it from `ServerConfig` through `AppConfig`. Do not add
   separate environment-variable and TOML implementations of the same settings.
2. Replace scattered hard-coded admission and deadline values with this type.
   Preserve a convenient default constructor for existing store tests; add a
   constructor that accepts limits. Test small limits using configuration,
   rather than opening tens of thousands of sockets in unit tests.
3. Use these initial defaults/candidate deployment values. They are admission
   ceilings, **not measured throughput or memory guarantees**:

   | Setting | Initial value | Meaning |
   | --- | ---: | --- |
   | `max_websockets` | 50,000 | Shared public/remote upgraded sockets |
   | `max_connections` | 60,000 | Shared HTTP + HTTPS accepted transports |
   | `max_public_requests` | 128 | API handler execution concurrency |
   | `max_internal_requests` | 32 | Internal handler execution concurrency |
   | `max_mutations` | 32 | Admitted queued + active mutations |
   | `max_response_bodies` | 1,024 | Ordinary HTTP responses still being sent |
   | `request_timeout_seconds` | 15 | Handler/body processing deadline |
   | `tls_handshake_timeout_seconds` | 10 | Existing rustls-acceptor behavior |
   | `header_timeout_seconds` | 10 | HTTP/1 header completion |
   | `write_stall_timeout_seconds` | 10 | No-progress write deadline |
   | `http_idle_timeout_seconds` | 60 | Idle ordinary HTTP connection |
   | `websocket_write_timeout_seconds` | 10 | Existing send deadline |

   Keep heartbeat interval/deadline at 30/90 seconds, inbound WebSocket messages
   at 4 KiB, JSON bodies at 2 MiB, and remote session limits unchanged. Keep the
   live message buffer bounded; package 2 fixes the burst behavior instead of
   increasing it to one million.
4. Reject zero/overflowing limits and inconsistent values such as a transport
   ceiling smaller than the socket ceiling. Share the transport budget across
   both listeners. Cloning routers must not create independent budgets.
5. Log effective non-secret limits once at startup. Add `Retry-After` to HTTP
   admission 429 responses where a short retry is meaningful. Browser WebSocket
   JavaScript cannot inspect failed-handshake response headers; package 5 must
   work without them.

Tests: omitted defaults, override parsing, invalid configuration, tiny admission
limits, release on failed upgrade/disconnect, and sharing across router clones.
Acceptance: no hard-coded 2,048 ceiling remains; settings support 40,000 sockets
plus headroom without changing application source.

## 2. Recover run-stream lag without disconnecting healthy viewers

Files: `server/server-v2/src/public.rs`, `server/service/src/membroadcast.rs`,
`server/service/src/event_store.rs`, `server/service/src/event_store/engine.rs`,
`server/server-v2/tests/one_shot_public.rs`.

Use the smallest fix first:

1. Match `RecvError::Lagged` separately from `Closed` in `runs_ws`. On lag,
   subscribe to the latest shared replay and continue on the same socket.
   Event deletion/channel replacement must still close the old stream so current
   deletion behavior is preserved. Do not turn `Closed` into an endless loop.
2. Read the current freeze policy when recovering and retain the existing checks
   before sending each run. Never reuse a pre-freeze unmasked serialized replay.
3. Keep the existing atomic subscription/snapshot relationship under the engine
   lock. Do not introduce a gap between capturing replay and subscribing to live
   changes. Preserve `Arc` sharing; no full-history clone per viewer.
4. Confirm the client updates runs by ID and corrections remain correct when a
   replay repeats existing IDs. Add a client-model test if current tests do not
   establish this. Do not assume delivery must be exactly once.
5. Keep heartbeat/read branches responsive while replaying. Bound outgoing writes
   and retain the slow-client disconnect behavior; recovery is not permission to
   retain an unlimited backlog. Count lag recoveries and replay work.

Tests: an open socket receives a batch of at least 2,000 changed runs and then a
sentinel update without reconnecting; corrections to existing IDs; a freeze
change during recovery; event deletion; a lagged timer receiver recovers the
latest timer where appropriate. Use both memory and SQLite fixtures.

If repeated replay prevents convergence under the capacity workload, the next
optimization is a **shared batch of changed runs per commit** with receivers
iterating its contents. Keep the on-wire message per run unchanged. Implement
that only with a reproducer showing why replay recovery is insufficient; do not
start by replacing the entire event engine.

## 3. Make ordinary reads consistently use committed memory

Files: `server/service/src/event_store.rs`,
`server/server-v2/tests/database_service.rs`, database/API documentation.

Implementation:

1. Hydrate and validate storage at startup as today. After initialization, all
   ordinary public **and internal** reads use the committed engine. Audit every
   `refresh()` caller: `get_event`, `get_contest`, `get_site`, `list_sites`,
   `contest_runs`, `revelation_urls`, `has_event`, and `get_team` are included.
   None of these reads should acquire the mutation gate or reload storage.
2. Keep writes serialized and publish only after durable commit. Mutations may
   still consult storage; do not silently change the database transaction model.
3. Separate startup hydration, ordinary reads, and ambiguous-write recovery.
   A failed read before a mutation has changed anything must leave the committed
   cache intact. Remove ordinary-read invalidation of the global initialized flag.
4. Preserve fail-closed handling of ambiguous commits: close affected streams and
   reconcile that event from authoritative storage. If reconciliation cannot read
   storage, mark that event unavailable (503), not nonexistent (404), until it
   can be reconciled. Keep other events available. Use a bounded, serialized
   recovery path with at most one pending recovery per event and retry backoff;
   do not spawn an unbounded task per public read. Successful recovery clears the
   marker. Event absence confirmed by storage may clear it as a genuine deletion.
5. Do not remove startup corruption checks or pretend an ambiguous write failed
   safely. External database edits/multiple writers remain unsupported.

Tests: initialize two events; inject read failure; both internal/public reads
continue from committed memory without increasing database-read counts; failed
pre-commit read preserves streams; committed writes become visible; startup
failure still fails; ambiguous commit reconciles; failed reconciliation affects
only its event and later recovers. Existing detached-failure/Sentry tests must
await explicit completion, not use a cached getter as a synchronization barrier.

## 4. Make write deadlines and retries honest

Files: `server/server-v2/src/resource_limits.rs`,
`server/service/src/event_store.rs`, `server/server-v2/src/openapi.rs`,
`server/server-v2/tests/database_service.rs`, internal API documentation.

Implementation decisions:

1. Move mutation-gate acquisition **before** spawning the detached commit task.
   Acquire an owned gate guard that can be moved into the task. While waiting
   for that guard, the caller remains cancellation-safe: timeout/disconnect drops
   its mutation admission permit and prevents the queued operation from starting.
   Once the task is spawned, retain the guard and permit through commit/publish.
   Keep request tracing and Sentry context attached.
2. Keep the handler deadline, but do not return an ordinary 408 for a timed-out
   mutating request. For POST/PUT/PATCH/DELETE timeout, return **503** with an
   explicit error code such as `operation_outcome_unknown` and a message that
   the operation may still commit. A conservative unknown result is acceptable
   even if timeout happened before mutation started. GET/HEAD timeouts may remain
   408. Admission rejection before processing remains 429.
3. Document that a timeout/transport failure is not proof a write failed; reconcile
   state before retrying non-idempotent commands. Do not add automatic mutation
   retries to the admin CLI. Do not claim exactly-once execution. Operation IDs,
   durable idempotency keys, and a new job API are outside this fix.
4. Preserve detached-task capacity limits even after callers disappear. A timeout
   must not permit unlimited work to accumulate behind a slow database.

Tests using `ControlledDatabase` and paused Tokio time: a queued timed-out write
never executes; a write already inside persistence can complete after timeout
and receives the explicit unknown-outcome response; successful writes keep their
existing responses; all permits are released; retrying admission does not bypass
the mutation bound. Test actual routed requests, not only the store helper.

## 5. Add browser reconnect backoff and stop abandoned streams

Files: `client-sdk/src/websocket_stream.rs`, `client-sdk/src/request.rs`,
`client-sdk/src/lib.rs`, `client-sdk/Cargo.toml`; browser integration tests.

Implementation:

1. Extract a small retry-policy helper with injectable time/random samples for
   deterministic unit tests. Use exponential backoff with jitter: initial base
   1 second, doubling to 30 seconds; choose each delay in `[base/2, base]`.
2. Reset WebSocket backoff only after a stable connection (30 seconds), not on
   opening the socket or receiving the first replay item. Repeated short-lived
   connections must not defeat backoff. Keep initial connection immediate.
3. Apply the policy to transport failures and transient HTTP overload as well as
   WebSocket reconnects. Audit both `request.rs` and the outer envelope retry
   loop in `lib.rs` so one fixed five-second loop does not bypass the policy.
   Respect usable HTTP `Retry-After` values with bounded delay handling. Preserve
   intentional before-start polling; do not treat a missing envelope `data` as
   success or break countdown-to-start behavior.
4. Stop and release a WebSocket task when its downstream receiver is dropped,
   including when the socket is idle. Merely checking a failed send is not enough.
   Navigation/unmount must not leave a reconnect loop consuming server slots.
5. Do not expose capability-bearing URLs in new logs. If retry logging touches
   existing URL logging, redact remote keys and query strings there as well.

Tests: delay range/cap/reset; many seeded clients get dispersed retry times;
short-lived opens do not reset; HTTP 429 recovery; canceled consumer closes an
idle connection and never reconnects; browser navigation and ordinary reconnection.
Build the WASM client; a native-only test does not validate browser integration.

## 6. Finish protections for direct HTTP/HTTPS serving

Files: `server/server-v2/src/lib.rs`, new transport module under
`server/server-v2/src/`, `server/server-v2/src/resource_limits.rs`, relevant Cargo
manifests, transport integration tests.

Verified implementation starting points at the reviewed dependency versions:
`axum-server 0.8.0` exposes `accept::Accept`, `Server::acceptor/map`, and
`Server::http_builder`. Its rustls acceptor **already defaults to a 10-second TLS
handshake timeout**. Inspect the installed source before coding; do not invent
builder methods or replace working TLS handling unnecessarily.

Implementation:

1. Prefer using `axum-server` for both plain HTTP and HTTPS so both listeners have
   one transport policy. Preserve cleartext rejection of `/internal`, ALPN,
   HTTP/2 support, upgrades, routing, and shutdown behavior.
2. Wrap the accepted IO with an owned shared connection permit **before TLS
   negotiation**. Reject excess transports promptly instead of awaiting permits
   in an unbounded queue. Keep the permit in the IO wrapper until actual close,
   including after a WebSocket upgrade; returning a 101 must not release it.
3. Explicitly configure the existing TLS timeout and HTTP/1 header deadline.
   Bound HTTP/2 concurrent streams and header sizes using supported builders;
   ensure HTTP/2 requests still pass through shared request/body admission.
   An HTTP/1 header timeout alone is not HTTP/2 protection.
4. Enforce a no-progress write timeout at the transport layer and a separately
   tested idle HTTP policy. Do not impose an absolute connection lifetime on
   WebSockets. Heartbeats must keep responsive idle browser sockets healthy.
   Avoid a read timer that incorrectly closes a response while useful output
   is being sent. Deadlines should measure the operation they actually protect.
5. Add a separate ordinary-response-body budget. Hold its permit inside a body
   wrapper until EOF/error/drop, not just until the handler returns `Response`.
   Include static assets and file-backed photos/sounds. Do not hold this HTTP
   body permit for the lifetime of an upgraded socket. Keep internal API
   execution admission separate from public execution admission.
6. Keep request-body size/deadline protection before mutation starts. Verify
   interrupted uploads never execute a partial operation. Bounded connection,
   HTTP/2 stream, handler, body, and mutation limits protect different resources;
   document those lifetimes rather than calling one semaphore a global limit.
7. Do not introduce a low per-IP socket cap: shared university/NAT addresses can
   legitimately contain many viewers. Do not trust client forwarding headers
   for identity on this direct-facing server.

Tests with small configured limits: both listeners share capacity; incomplete TLS
and headers expire; excess transports do not grow a waiting queue; HTTP/2 streams
cannot evade admission; slow uploads never mutate; slow readers release body and
transport capacity; static responses are included; failed upgrade releases slots;
successful upgrade retains its transport slot; browser sockets remain alive across
multiple heartbeat intervals and release all permits after close; internal HTTPS
and cleartext rejection still work. Run local socket tests outside a sandbox that
prohibits binding sockets, if necessary.

## 7. Add observability and a repeatable capacity test

Files: `server/server-v2/src/metrics.rs`, admission/transport/store/stream modules;
new `tools/viewer-load/` Rust tool and `doc/capacity-testing.md`.

Use the existing metrics/exporter stack. Add aggregate gauges/counters for active
transports, WebSockets by stream type, in-flight handlers, response bodies,
mutations, rejected admissions, lag recoveries, replayed messages, write stalls,
heartbeat disconnects, recovery attempts, and write outcomes. Add mutation and
request duration histograms. Never label metrics by key, raw URL, IP, or unbounded
event/contest identifiers. Account correctly on cancellation/error/drop paths.

Build a load tool using existing Tokio/WebSocket dependencies. It may emulate
browsers for scale: consume frames, flush automatic pong replies, open two public
sockets per logical viewer, and validate current run IDs/corrections/timers. Use
actual headless browsers for a small compatibility test; 20,000 browser processes
are unnecessary. Keep per-connection load-generator state small and validate its
own CPU/memory/network capacity. Support multiple independent generator processes
and source hosts to avoid ephemeral-port exhaustion and target-host contention.

Tool inputs: target URL, event/contest, viewer count, ramp duration, hold duration,
random seed, reconnect scenario, and output report path. TLS verification must be
on by default, with an explicit test CA option. Use separate feeder/admin tooling
to seed and update data; never put credentials into printed command lines/reports.

Required workload (generic synthetic names; no organization-specific examples):

1. Release build, production-equivalent host/network/TLS and the intended database
   backend. Record commit, CPU, RAM, NIC, OS, limits, dataset, and generator hosts.
2. Seed 2,000 teams and 10,000 historical runs. Record actual JSON sizes. The
   synthetic dataset is a reproducible minimum, not a substitute for the real
   largest expected event; repeat with that event's scale if larger.
3. Ramp to 20,000 viewers over 5 minutes; hold 60 minutes. Publish timer updates
   once per second and 10 new/corrected runs per second. Measure initial replay
   completion, live latency, active sockets, CPU, RSS, FDs, bandwidth, and errors.
4. Publish a 2,000-run correction batch while all viewers are connected. Change
   freeze boundaries during traffic and verify answer masking and convergence.
5. Disconnect/reconnect 25% of viewers together, then repeat with all viewers.
   Exercise jittered retries and fresh replay. Keep feeder/admin requests running.
6. Freeze the timer and verify idle browser connections survive. Perform an
   eight-hour smaller soak or equivalent time-controlled heartbeat tests plus
   the full-load 60-minute run; record which was actually executed.
7. Exercise slow readers/uploads and deliberate capacity exhaustion in a separate
   bounded scenario. Verify recovery after pressure stops. Do not mix expected
   overload failures into the clean-load success statistics.

Initial acceptance targets for the dedicated test environment:

- 40,000 healthy public sockets remain established in the clean steady phase;
  no capacity-induced disconnects, unexpected 429/503, or missing latest run state.
- p99 live timer/run update latency under 2 seconds; p99 normal internal mutation
  response latency under 2 seconds during steady viewer load.
- Initial ramp participants finish replay within 60 seconds of opening their
  socket; all clients converge after the correction batch within 30 seconds.
- After a full intentional disconnect, 99% of viewers reconnect and finish replay
  within 5 minutes, all within 10 minutes, with feeder writes continuing.
- RSS/FD counts stabilize after warmup and return near baseline after disconnect;
  no OOM, growing task backlog, or leaked admission permits. Retain at least 20%
  memory and sustained CPU headroom on the measured host.

These are explicit engineering targets for this plan, not claims about the current
server. If they fail, profile before optimizing. First examine per-viewer run
cloning/serialization, per-run timer-lock access, replay amplification, synchronous
work while holding the engine write lock, and fan-out allocation. Shared prepared
messages are allowed only when correctly keyed/invalidation-tested for contest
filtering and freeze policy. Do not cache unmasked answers into public messages.

A small automated smoke load belongs in CI. The 20,000-viewer capacity run belongs
on dedicated hardware. If such hardware is unavailable, deliver the tested tool
and commands, mark capacity validation pending, and state exactly what is missing.

## 8. Align deployment examples, docs, versions, and final checks

Files: native server units under `config/nacional_2026/` and
`config/regional_2026/`, `docker-compose.yaml`, `config/colombia/compose.yaml`,
server TOML examples, `doc/databases.md`, `doc/public-api-overview.md`,
`doc/internal-api-setup.md`, `doc/event-api.md`, `doc/nginx-examples/README.md`,
`CHANGELOG.md`, workspace manifest/lockfile, OpenAPI snapshots.

Implementation:

1. Document direct Axum as a supported production topology; nginx examples remain
   optional. Remove native-unit instructions that require a nonexistent private
   proxy. Distinguish direct host deployment from loopback-only Compose examples.
   Do not silently expose existing development Compose ports or change real host
   firewall rules as part of this work.
2. Remove the unmeasured 2-core/2-GiB restriction from the production-oriented
   native examples; provide explicitly documented, measured limits in the capacity
   profile/report. Retain configurable resource protection. Do not replace those
   numbers with an equally unsupported promise about another RAM/CPU allocation.
   Increase example `LimitNOFILE` to 131,072 for transport/reconnect headroom and
   explain that this is a descriptor budget, not a viewer benchmark result.
3. Fix the paragraph saying remote keys are arbitrary identifiers. Update limits,
   lag recovery, read-cache behavior, timeout outcomes, retry guidance, and direct
   transport protection in served OpenAPI descriptions as well as Markdown.
4. Keep one version source: workspace `Cargo.toml`. The plan file alone needs no
   API bump. For implementation based directly on reviewed 5.0.0, use **6.0.0**
   because the timeout/status and resource behavior change. If the base has moved,
   apply `AGENTS.md` to that actual base instead of downgrading or reusing a version.
   Update workspace packages in `Cargo.lock` and describe the final combined
   behavior in the changelog. Do not create/tag/push a release under this plan.
5. Capture the implementation starting commit as `IMPLEMENTATION_BASE` before
   editing. Run compatibility checks both against `v2.4.0` (migration review) and
   that actual base (new work). Never change the baseline to regenerated snapshots.

Final commands (use the repository's configured environment):

```bash
cargo fmt --all -- --check
make test-databases
cargo test --locked -p client-model -p client-sdk -p offline-packager
# Run the new transport, retry, browser, and load-smoke checks from their README.
make rebuild-client-for-release
make api-snapshots
make api-check API_BASE=v2.4.0
make api-check API_BASE="$IMPLEMENTATION_BASE"
python3 -m unittest discover -s scripts/tests
git diff --check
```

Use the mandated Docker image `tufin/oasdiff:v1.30.0`. If the local compiler cache
wrapper is blocked, use `RUSTC_WRAPPER=` rather than changing project configuration.
If dependency manifests change, run the repository dependency checks as well.
Inspect the final diff and verify no secrets, generated load datasets, or temporary
test files are committed. API specs, version, lockfile, and changelog travel together.

## Execution checklist and handoff

- [x] 1. Validated configurable limits, no 2,048 hard ceiling.
  Implemented defaults/config parsing, startup validation, shared router request
  budgets and configurable store admission. Transport-specific settings are
  declared now and will be enforced in package 6. WebSocket writes retain 10s.
  Checks: configuration/permit regressions, documented database suites, OpenAPI
  comparison against implementation base; no capacity claim yet.
- [x] 2. Batch/lag recovery preserves connections and freeze correctness.
  Runs recover through shared replay; timers atomically replace lagged receivers.
  Regression: 2,000-run batch and later correction on the same socket. Existing
  freeze tests retain masking coverage; no new wire message shape.
- [x] 3. Committed reads survive storage outages; recovery is scoped.
  All ordinary getters use committed memory. One retry worker reconciles only
  ambiguous events with backoff, retaining mutation admission and serialization.
- [x] 4. Queued cancellation and unknown write outcomes tested through HTTP.
  Mutation gate acquisition precedes detachment; writing deadlines return explicit
  503 unknown outcomes. Controlled storage tests cover cancellation, late commit,
  scoped failure and recovery.
- [x] 5. Browser retry jitter, stable reset, and abandoned-stream cleanup.
  Cancel-on-drop subscriptions propagate cancellation through conversion tasks.
  Browser WebSocket attempts await open before measuring stable lifetime; HTTP
  and envelope retries use jitter and bounded Retry-After. Native SDK/model tests,
  full Trunk release build, and headless Chrome idle-cancellation regression pass.
- [ ] 6. Direct HTTP/HTTPS transport/body protections and lifecycle tests.
- [ ] 7. Metrics, load tool, browser smoke, and saved capacity results.
- [ ] 8. Deployment/docs/version/API checks complete.
- [ ] Dedicated 20,000-viewer run meets acceptance targets, or is explicitly pending.

For each completed package record the changed files, checks run, result, and any
remaining constraint. The final handoff must distinguish implemented limits,
automated correctness tests, measured capacity, and production deployment status.
