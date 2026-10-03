# Accepted dependency maintenance advisories

Reviewed on 2026-10-03 after updating workspace dependencies to current stable
releases. The following maintenance notices remain in the dependency graph and
are accepted individually in `deny.toml`. They describe abandoned maintenance;
these exceptions do not assert that the crates are risk-free or exempt future
security advisories for the same crates.

| Advisory | Locked crate | Dependency path | Reason retained |
| --- | --- | --- | --- |
| [RUSTSEC-2024-0388](https://rustsec.org/advisories/RUSTSEC-2024-0388.html) | derivative 2.2.0 | data → derivative | No maintained release; keep existing data model derives rather than refactoring them solely for this notice. |
| [RUSTSEC-2024-0436](https://rustsec.org/advisories/RUSTSEC-2024-0436.html) | paste 1.0.15 | Leptos 0.8.21, leptos-use 0.19.2, either_of, reactive_graph, reactive_stores, and tachys → paste | Current framework releases still require it; no framework forks or vendored patches are introduced. |
| [RUSTSEC-2026-0173](https://rustsec.org/advisories/RUSTSEC-2026-0173.html) | proc-macro-error2 2.0.1 | Leptos macro/hot-reload dependencies → rstml 0.12.1 → syn_derive 0.2.0 → proc-macro-error2 | Still required upstream, even after updating the Leptos macro crates. |
| [RUSTSEC-2025-0134](https://rustsec.org/advisories/RUSTSEC-2025-0134.html) | rustls-pemfile 2.2.0 | service → rustls-pemfile | No maintained release; keep the existing certificate/private-key loader rather than migrating its parsing API solely for this notice. |

The update replaces vulnerable rustls 0.23.43 with 0.23.45, resolving
[RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285.html), and
replaces yanked chacha20 0.10.1 with 0.10.2. Neither receives an exception.

`make cargo-deny` continues checking vulnerabilities, yanked releases, dependency
bans, and unknown registry/Git sources across all workspace features. It reports
the accepted advisory IDs as notes. Existing duplicate-version and local path
wildcard warnings remain enabled. License checking remains outside this policy.

On each dependency refresh, rerun `cargo tree -i <crate>` and `make cargo-deny`,
review the upstream advisories, and remove exceptions whose dependency has
disappeared. New advisory IDs require separate review; do not disable maintenance
checks globally. The weekly CI run refreshes RustSec independently of dependency
updates and continues failing on findings outside these four accepted IDs.
