# Manage events with animeitor-admin

`animeitor-admin` manages a running server through its internal HTTPS API. It reads `server.toml` by default; override with `--server-config PATH`. It uses `server_url`, the enabled token selected by `client_token`, and `tls_ca_cert`. Relative certificate paths resolve against the configuration file. It does not modify the configuration file, start a server, or run a live clock.

Build and inspect commands:

```sh
cargo build -p cli --bin animeitor-admin
target/debug/animeitor-admin --help
target/debug/animeitor-admin events update --help
```

All examples below assume `animeitor-admin` is on PATH (otherwise use `target/debug/animeitor-admin`). Global `--server-config` and `--json` work before or after subcommands.

## Create and prepare an event

Create a JSON roster, then supply routine settings as flags:

```sh
cat > teams.json <<'JSON'
[{"login":"teambr001","escola":"Example University","nome":"Example Team"}]
JSON

animeitor-admin events create regional-2026 \
  --problem A --problem B --teams-file teams.json \
  --score-freeze-time-seconds 14400 --penalty-seconds 1200 --time-seconds -60
animeitor-admin contests create regional-2026 brasil --code '^teambr'
animeitor-admin sites create regional-2026 brasil campus --code '^teambr001$'
animeitor-admin revelation-urls regional-2026
```

The event name comes from the positional identifier. All times use seconds. Omitting `--time-seconds` on creation defaults to **zero**, opening public contest data immediately. The server does not advance the timer automatically.

`revelation-urls` reads current server state and lists every site's full private URL. The `secret` query parameter is also the site's Bearer token for `runs_secret`. Keep these URLs private and fetch them again after salt rotation. The offline `printurls` command remains available for links derived from configuration files.

## Resource commands

| Resource | Commands and positional identifiers |
| --- | --- |
| Events | `events list`; `events get/create/update/replace/delete/salt EVENT` |
| Contests | `contests list EVENT`; `contests get/create/update/replace/delete/salt EVENT CONTEST` |
| Sites | `sites list EVENT CONTEST`; `sites get/create/update/replace/delete/salt EVENT CONTEST SITE` |
| Teams | `teams list EVENT`; `teams add EVENT`; `teams get/update/delete EVENT LOGIN` |
| Problems | `problems list EVENT`; `problems add/delete EVENT PROBLEM` |

Event flags: repeatable `--problem`, `--teams-file` (JSON array or `-` for stdin), `--score-freeze-time-seconds`, `--penalty-seconds`, `--time-seconds`, and `--salt`.

Contest flags: repeatable `--code`, `--gold`, `--silver`, `--bronze`, `--style`, `--photo-url-format`, `--sound-url-format`, and `--salt`. Medal flags map to the API's `ouro`, `prata`, and `bronze` fields. Site flags: repeatable `--code` and `--salt`. Regexes use Rust syntax; quote them to prevent shell expansion.

Create and update accept either field flags or `--file PATH` (`-` for stdin). JSON is a bare resource object, never a `data` envelope. Missing `name` is filled from positional identifiers for creation/replacement; a conflicting name is rejected. Event creation requires a problem list, team list, freeze time, and penalty. Contest/site creation requires codes; JSON permits explicitly empty arrays. Unspecified optional creation values use the server defaults.

`update` sends **one atomic PATCH**, preserving omitted fields. Arrays supplied in a patch replace the whole array. `replace --file PATH` sends a full PUT, resetting omitted optional values. Unknown fields and invalid types are rejected locally. No automatic read-modify-write or mutation retry is performed.

```sh
animeitor-admin contests update regional-2026 brasil --gold 4 --silver 8 --bronze 12
animeitor-admin contests update regional-2026 brasil --unset style --unset photo_url_format
printf '%s\n' '{"salt":null,"sound_url_format":null}' | \
  animeitor-admin contests update regional-2026 brasil --file -
animeitor-admin events get regional-2026 --json
```

Use `--unset` with API field names for nullable fields: `salt`, `style`, `photo_url_format`, and `sound_url_format` where applicable. Set/unset conflicts are rejected; JSON null has the same clearing meaning. `--file` cannot be combined with field flags or `--unset`. Required fields cannot be cleared.

## Incremental roster, problems, and filters

```sh
animeitor-admin teams add regional-2026 --login teambr002 --escola 'Another University' --nome 'Second Team'
animeitor-admin teams update regional-2026 teambr002 --nome 'Updated Team'
animeitor-admin problems add regional-2026 C
animeitor-admin contests codes regional-2026 brasil --add '^teambr002$' --remove '^oldteam$'
animeitor-admin sites codes regional-2026 brasil campus --add '^teambr002$'
```

Team add/update also accept `--file PATH`. Team login is immutable. Problem addition appends to display order; reorder through an event update with a full `problems` array. Filter deltas compare exact regex strings, preserve retained order, and append additions. Adding existing patterns/removing absent patterns is a no-op; supplying the same pattern in both lists is invalid.

Deletion is explicit and noninteractive:

```sh
animeitor-admin teams delete regional-2026 teambr002
animeitor-admin teams delete regional-2026 teambr002 --keep-runs
animeitor-admin problems delete regional-2026 C
```

Deleting a team or problem with stored runs returns `409 conflict`. `--keep-runs` permits team deletion while retaining submissions and replay history. It also works with `events update --file roster-patch.json --keep-runs` when replacing `teams`. Retained runs may still appear in regex-based streams; recreating the login associates them with the team again. There is no corresponding override for problems. Existing legacy full PUT replacement remains available and retains its old semantics, without the incremental reference checks.

Event deletion removes its contests, sites, and runs. Contest deletion removes its sites but retains event runs. Site deletion retains runs. Commands do not ask for confirmation.

## Runs, timer, salts, and metrics

```sh
animeitor-admin runs add regional-2026 --id 1 --team-login teambr001 --problem A --time-seconds 56 --answer Y
animeitor-admin runs delete regional-2026 1
animeitor-admin runs import regional-2026 --file runs.json
animeitor-admin timer set regional-2026 --seconds -120
animeitor-admin timer set regional-2026 --seconds 0
animeitor-admin sites salt regional-2026 brasil campus
animeitor-admin contests salt regional-2026 brasil --salt new-value
animeitor-admin events salt regional-2026
animeitor-admin metrics
```

`runs delete EVENT ID` removes one submission (204), or returns 404 if the event or ID does not exist. Existing run streams close and reconnect with the remaining submissions. A later import can recreate the deleted ID.

Run answers are `Y`, `N`, `?`, or `X`. Import accepts `{"runs":[...]}` via file/stdin. Existing IDs are corrected; identical resends are no-ops. Unknown teams produce warnings rather than rejecting a batch. `runs clear EVENT` clears stored submissions, **but existing WebSocket replay history remains**; recreate the event and its configuration for a clean stream history.

Salt commands without `--salt` generate a random value. Rotation invalidates links for the affected scope; fetch `revelation-urls` again. Clear a salt with an update using `--unset salt` instead.

The timer command sets a value once. A feeder/controller must send further updates. The feeder's existing source-driven behavior is unchanged and can overwrite manual edits. Coordinate changes with the source configuration. Existing run streams capture filters and freeze boundaries; reconnect after modifying those settings.

## Output, errors, and Docker

Readable output is the default. `--json` writes the API envelope to stdout, preserving warnings. Team/problem list commands select the relevant event array into `data`. Successful `204` operations have no JSON output. Metrics are raw Prometheus text, or `{"data":"..."}` with `--json`.

Readable-mode warnings go to stderr; errors always go to stderr. Exit status is 0 for success including warnings, 1 for input/configuration/network/API failures, and 2 for Clap argument-usage errors. Requests time out after 30 seconds. Errors report HTTP status in readable mode and preserve structured API errors in JSON mode. Private configuration contents are not included in configuration parse errors; no request/response payloads are logged.

The Docker image includes `/animeitor-admin`. To use the running example Compose server (which already has the config and CA mounted):

```sh
docker compose exec animeitor /animeitor-admin --server-config /workspace/server.toml events list
docker compose exec animeitor /animeitor-admin --server-config /workspace/server.toml revelation-urls regional-2026
```

For host usage, `server_url` must resolve from the host (typically `https://localhost:8443`); the Docker-only `animeitor` hostname is intended for containers on the Compose network. Use an image rebuilt from this revision to obtain the new binary.
