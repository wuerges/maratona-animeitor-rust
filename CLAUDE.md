# maratona-animeitor

Scoreboard/animeitor for ICPC-style contests. Single cargo workspace at the root.

- `client-v2/` — Leptos 0.8 CSR (wasm) UI, built with `trunk` from that dir (`make rebuild-client-for-release`, dev: `trunk serve`). Nightly toolchain pin lives in `client-v2/rust-toolchain.toml`.
- `client-model/` — reactive contest state (signals); depends only on `reactive_graph` + `data`. No leptos/IO — platform-neutral for future clients.
- `client-sdk/` — HTTP/WS layer (gloo-net, wasm-only for now); plain types, no signals; native backend can be cfg-gated in later. Client URL config is runtime: relative same-origin defaults, overridden by a `config.json` next to index.html (fetched at startup by `SdkConfig::load`).
- `server/` crates — `data` (shared types, wasm-safe), `service`, `server-v2`, `cli` (binaries incl. `simples`). Build: `cargo build -p cli`, release+musl: `make rebuild-server-for-release`.
- `config/` — per-contest TOML configs.

## Client performance (critical)

The client renders ~1500 team rows, updating every second. Avoid re-rendering: fine-grained per-row signals (one signal per team, not one big list signal), memoize derived values. Any whole-list re-render costs 1500x. Review `client-model/` for the signal-per-team pattern before adding new reactive state.

## VCS

Uses **jj**, not git. Commit each change as you go (`jj commit`); `jj status`/`jj diff` for status/diff.
