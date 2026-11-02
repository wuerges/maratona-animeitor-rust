# Direct Axum capacity testing

The target is 20,000 simultaneous browser viewers on one server process: 40,000
public WebSockets. Axum serves HTTP, HTTPS, static assets, and upgrades directly.
Nginx is optional. The configured ceilings are not a measured capacity guarantee.
**The dedicated 20,000-viewer capacity run is pending.** Local tests establish
correctness of admission, TLS/HTTP/2, replay, cancellation, and the load tools.

## Server profile and host preparation

Use a release build and the same TLS, storage backend, assets, event size, and
network path intended for production. One server process owns the database.
Retain internal HTTPS authentication. Use a dedicated event and token whose event
pattern permits `capacity-test`. The workload publisher refuses to replace an
existing event; it never deletes an event automatically.

The optional server TOML limits default to:

```toml
[limits]
max_websockets = 50000
max_connections = 60000
max_public_requests = 128
max_internal_requests = 32
max_mutations = 32
max_response_bodies = 1024
request_timeout_seconds = 15
tls_handshake_timeout_seconds = 10
header_timeout_seconds = 10
write_stall_timeout_seconds = 10
http_idle_timeout_seconds = 60
```

Transport slots are shared across HTTP/HTTPS and retained through upgrades. HTTP
idle expiry applies to ordinary connections; upgraded sockets retain 30-second
ping / 90-second matching-pong liveness. Browsers answer pings automatically.
Incoming WebSocket messages remain bounded to 4 KiB; JSON bodies to 2 MiB. Output
writes retain a 10-second deadline. HTTP/2 has 128 concurrent streams per transport
and a 32-KiB header-list bound. Ordinary response-body admission starts before
handlers, covers static/media assets, and lasts until EOF/error/drop. Successful
upgrades immediately release their temporary HTTP response admission.

Native example units use `LimitNOFILE=131072`. Their earlier two-core/2-GiB values
were unmeasured and have been removed from the production-oriented examples.
Configure CPU/memory limits in a systemd drop-in using actual measurements with at
least 20% headroom; record them with the report. Keep process/thread and descriptor
bounds. Demo Compose files retain their smaller host budgets and loopback ports;
those files are not a validated 20,000-viewer deployment profile. Do not apply
loopback-only or proxy-only firewall instructions to a direct public listener.

Monitor the server and every load host. Use multiple generator hosts/processes
when needed: 40,000 outbound sockets to one address can exhaust ephemeral ports
on a single source. Each runs consumer keeps a bounded ID-to-fingerprint map
(default maximum 100,000 IDs), so generator memory grows with viewers × replay
size. Distribute viewers and measure generator CPU/RSS/network; a saturated
load generator does not establish a server limit. Match the bound to the dataset.
No low per-IP viewer cap is imposed; shared campus/NAT addresses are supported.

## Build and seed

From the repository root:

```bash
cargo build --release -p server-v2 -p cli -p viewer-load
python3 scripts/capacity_workload.py seed \
  --server-config /path/to/dedicated-server.toml \
  --event capacity-test --contest all --teams 2000 --runs 10000 \
  --expected /tmp/capacity-expected.json --ledger /tmp/capacity-publish.jsonl
```

The server must be running on the dedicated target. The publisher reads the
selected credential and CA from the server TOML file, verifies HTTPS, and does not
print credentials or retry mutations. It creates 2,000 teams and 10,000 runs by
default. It stores a local publisher-state file beside the expectation file; keep
that file for subsequent publisher commands. It is the single writer for this
dedicated synthetic event. Do not modify runs outside the publisher during a test.
A mutation timeout is an unknown outcome; reconcile it before continuing. Use a
new dedicated event if the publisher's local state cannot be reconciled.

The expectation JSON contains exactly the latest masked public runs and timer:

```json
{"runs":[{"id":1,"team_login":"team00001","prob":"A","time_seconds":10,"answer":"Y"}],"timer":{"current_time_seconds":60,"score_freeze_time_seconds":999999}}
```

The publisher replaces this file atomically after acknowledged updates. The load
tool watches it centrally, shares expectations across consumers, and checks final
state including corrections and masking. Finish publisher writes at least two
seconds before the load tool stops so the final expectation is stable. Copy or
synchronize the expectation file to each generator host atomically when they are
on different hosts. The publisher-state file contains synthetic raw answers and
is not suitable for distribution to real public clients.

## Run the workload

On generator hosts, split `--viewers` so their total is 20,000. Use different seeds.
The target is the server origin (or an explicit deployment prefix), without `/api`.
A custom test CA is supplied with `--ca`; without it, native trust roots are used.
TLS verification is always enabled. Never use a certificate-bypass switch.

```bash
target/release/viewer-load \
  --target https://dedicated.example.com \
  --event capacity-test --contest all --viewers 20000 \
  --ramp-seconds 300 --hold-seconds 3600 --seed 101 \
  --expect-file /tmp/capacity-expected.json \
  --trace /tmp/capacity-receives.jsonl --sample-viewers 100 \
  --output /tmp/capacity-viewers.json
```

Each viewer uses timer and runs sockets, consumes browser-style control frames,
and flushes pong replies. The report records peak sockets, one-second active
socket samples, opened sockets, failed attempts, unexpected disconnects, initial
replay completion, invalid messages, and final expectation mismatches. Trace
output is bounded by a channel and sampled by viewer; dropped trace records make
the run fail instead of silently producing a latency claim. Trace disk space is
not capped; provision storage for the selected duration/sample count.

The tool exits unsuccessfully on task errors, invalid messages, final mismatches,
unexpected disconnects, trace drops, or failure to reach the requested socket
peak. Inspect sustained samples and server metrics as well: the peak alone cannot
establish a successful hold. No expectation file means connectivity-only checks,
not state-correctness validation.

While viewers hold, run the publisher. This publishes 10 runs and one timer update
per second; the example also sends a 2,000-run correction batch and changes freeze
policy during the same live traffic. Run it a little shorter than the load hold:

```bash
python3 scripts/capacity_workload.py live \
  --server-config /path/to/dedicated-server.toml \
  --event capacity-test --contest all --teams 2000 --seconds 3500 \
  --correct-at 600 --freeze-at 1200 --freeze-seconds 0 \
  --expected /tmp/capacity-expected.json --ledger /tmp/capacity-publish.jsonl
```

For separate phases, modes `correct`, `freeze`, and `snapshot` use the same files.
Keep exactly one publisher active. A frozen timer idle phase means stop timer
updates and keep viewers connected; verify heartbeats retain healthy sockets.
Repeat the viewer run with `--reconnect quarter` and `--reconnect all`. The selected
viewers deliberately disconnect halfway through the hold and reconnect with
seeded jitter. These scenarios still fail on *unexpected* disconnects. Keep
publisher/admin traffic active and inspect sustained recovery in the report.

For slow readers/uploads or deliberate admission exhaustion, use a separate
bounded test with small configured budgets. Do not mix expected overload failures
into the clean-load success statistics. Unit/socket lifecycle tests cover these
paths automatically; dedicated adversarial capacity testing remains a separate
measurement task.

## Latency and resource evidence

Publisher and generator hosts must have synchronized clocks. Publisher ledger
records include request-send timestamps and public message values. Sampled viewer
traces record receipt timestamps for changed messages after initial replay;
unchanged duplicates and initial timer frames are excluded. Freeze publications
are included in the ledger. Analyze their join after the run:

```bash
python3 scripts/analyze_viewer_latency.py \
  --ledger /tmp/capacity-publish.jsonl --trace /tmp/capacity-receives.jsonl \
  --requests /tmp/capacity-publish.requests.jsonl
```

The result measures request-send-to-receive latency, including mutation processing.
Unmatched initial replay is reported separately. Negative intervals fail the
analysis and require clock investigation. Repeated identical values match the
latest preceding publication. The sibling `.requests.jsonl` publisher log records
HTTP status/method and request duration without credentials; `--requests` computes
p99 internal mutation response latency from it. Archive traces/ledger so independent
reviewers can reproduce percentiles. Multiple generator traces can be analyzed
separately, or concatenated for an overall sampled distribution.

Authenticated `/internal/metrics` exposes existing request/storage metrics plus:

- `server_resources_active{kind=...}` for transport, runs/timer/remote sockets,
  HTTP handlers, response bodies, and admitted mutations.
- `server_resource_lifetime_seconds{kind=...}` histograms, including queued plus
  executing mutation lifetimes and handler duration.
- `server_resource_events_total{kind=...}` for admission rejection, lag recovery,
  stream output, write/heartbeat expiry, recovery attempts and write outcomes.
- `server_replay_messages_total` for replay messages offered to subscribers.

Labels have fixed resource/event categories; no capability, raw URL, IP, or
unbounded event identifier is used. Capture metrics alongside RSS, CPU, open FDs,
NIC throughput, TCP socket memory, and generator utilization. Confirm all resource
gauges and FDs return toward baseline after clients leave. Some lifetime counters
include failed/canceled work intentionally.

## Acceptance and reporting

Record commit, configuration, actual CPU/RAM/NIC/OS, storage backend, dataset sizes
and JSON sizes, load host details, certificate/trust configuration, and every
command. Keep server builds and expected/actual reports identifiable. Targets:

- 40,000 healthy public sockets throughout the clean steady phase; no unexpected
  errors/disconnects, missing state, or false freeze answers.
- p99 live message and ordinary internal mutation latency below 2 seconds.
- p99 initial replay completion within 60 seconds; all viewers converge within
  30 seconds after the correction batch.
- 99% reconnect and finish replay within 5 minutes after a full intentional
  disconnect, all within 10 minutes, with feeder writes continuing.
- RSS/FD/task counts stabilize, no OOM or leaked admission, and at least 20%
  memory and sustained CPU headroom. Perform an eight-hour smaller soak or use
  the explicit time-controlled heartbeat evidence and state its limitation.

The load report currently records maximum initial replay duration, a stricter
bound than p99 if all expected replays complete. Check completion count as well.
Reconnect convergence is assessed from connection samples plus final-state checks;
for precise state convergence times, correlate sampled traces with the publisher
ledger. Report the sampling coverage instead of claiming every client was timed.

Local automated evidence: native database/API suites; socket tests for HTTP/TLS,
HTTP/2, upgrades and deadlines; WASM release build; headless Chrome cancellation and full application/export test;
small workload reconnect/correction smoke; verified HTTPS publisher seed/live/
correction/freeze smoke. CI runs the small workload; it does not run 20,000 viewers.
No production deployment or dedicated capacity result is implied by these checks.
