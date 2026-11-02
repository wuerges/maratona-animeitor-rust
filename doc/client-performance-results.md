# Client performance validation

Measured on 2026-11-02 using release WebAssembly builds, headless Chromium
156.0.8078.47 on Linux x86_64, Intel Core i9-12900HK. Baseline: `becce025`.
Both datasets have 15 problems and 100,000 deterministic submissions, including
frozen submissions and a live correction. Tests use a local fixture server.

## Results

| Measurement | Previous, 2,000 teams | Updated, 2,000 teams | Updated, 5,000 teams |
| --- | ---: | ---: | ---: |
| Mounted scoreboard rows during scrolling | 2,000 | 18–29 | 18–29 |
| DOM nodes during scrolling | 141,948 | 4,657–5,380 | 4,307–4,978 |
| Reveal forward response, p95 | 599 ms | 34 ms | 31 ms |
| Reveal backward response, p95 | 444 ms | 28 ms | 30 ms |

The previous build mounted all 5,000 rows (306,200 DOM nodes) and timed out
waiting for a scrolling command after 180 seconds. The updated build completed
the full 2,000/5,000-team workload in 41 seconds (including the offline regression suite). Ranking transitions temporarily
retained additional rows (38 mounted in the revelation measurements).

The final run measured p95 scrolling frame intervals of 16.8 ms at both sizes
(60 frames each). Both real-browser suites passed, including custom row heights,
media across scrolling, offline export, keyboard controls, filters, and reset.
All 21 client/model unit tests and the release WebAssembly build passed.

Initial display in the updated build took 1.9 seconds at 2,000 teams and 1.8
seconds at 5,000 teams. These are individual navigation-to-first-row measurements,
not stable startup benchmarks. The old build's startup measurements varied greatly
and should not be used to calculate a startup speedup.

Response samples measure keyboard dispatch through the next animation frame
(30 forward steps and 30 backward steps after Top 100). They include browser
scheduling and are not isolated scoring CPU timings. Browser JS heap samples omit
WebAssembly linear memory; they do not establish a total-memory bound. These
results do not establish p95 live batch latency or long-task rates under a sustained
production stream, or performance on lower-powered hardware.

## Reproduce

Build from `animeitor-client`:

```sh
NO_COLOR=true RUSTC_WRAPPER= trunk build --release --dist /tmp/client-performance-dist
```

From the repository root, using matching browser/driver binaries:

```sh
RUSTC_WRAPPER= \
OFFLINE_CLIENT_DIST=/tmp/client-performance-dist \
OFFLINE_BROWSER=chrome \
OFFLINE_BROWSER_BINARY=/path/to/chrome \
CHROMEDRIVER=/path/to/chromedriver \
cargo test -p offline-packager --test browser -- --ignored --nocapture

RUSTC_WRAPPER= cargo test -p client-model -p animeitor-client --lib
```

Use `CAPACITY_BASELINE=1` to disable the mounted-row bound assertion for an older
client. Build older sources with a separate target directory to avoid Cargo
artifact reuse between source trees. The browser test prints scroll extents, row
and node counts, JS heap samples, scrolling frame timings, and reveal timings.
It also exercises custom row heights and media selection across scrolling.

## Behavior and limits

Scoreboard rows use one viewport of overscan in each direction. Visible departures
remain mounted for the five-second transition; distant entrants start at the
viewport boundary. CSS `@starting-style` supplies the entry transition in modern
browsers; older browsers can display entrants immediately at their target position.
Submission panel transitions are independent. Photos/audio use a single selected
team overlay and survive row unmounting.

Revelation keeps up to eight checkpoints spaced 100 steps apart. Recent backwards
steps replay fewer than 100 steps. Backtracking beyond the retained history falls
back to the starting state; reset clears all checkpoints. Snapshots exclude raw
run history, but their scoring state still grows with competition size.
