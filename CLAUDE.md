# maratona-animeitor

Scoreboard/animeitor for ICPC-style contests. Rust:

- `client-v2/` — Leptos 0.8 CSR (wasm) UI, built with `trunk` (`make rebuild-client-for-release`, dev: `trunk serve`).
- `server/` — workspace (`data`, `cli`, `service`, `server-v2`): API + serves client release, photos, sounds. Standalone: `cargo run -p simples` (see Makefile).
- `config/` — per-contest TOML configs.

## Client performance (critical)

The client renders ~1500 team rows, updating every second. Avoid re-rendering: fine-grained per-row signals (one signal per team, not one big list signal), memoize derived values. Any whole-list re-render costs 1500x. Review `client-v2/src/model/` for the signal-per-team pattern before adding new reactive state.

## VCS

Uses **jj**, not git. Commit each change as you go (`jj commit`); `jj status`/`jj diff` for status/diff.
