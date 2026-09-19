# Configure an Animeitor event

This specification is a self-contained guide for an operator or agent configuring an **already running server**. All configuration operations are under `/internal`. The public API is described at `/api/openapi.json` (Swagger: `/api/docs`); this specification is served at `/internal/openapi.json` (Swagger: `/internal/docs`). Both internal documentation endpoints require authentication.

## Prerequisites and conventions

Obtain the internal **HTTPS base URL**, a configured **username and its token**, and the event's problem list, team roster, timing, contest groups, and site groups. Use HTTP Basic authentication (`Authorization: Basic base64(username:token)`); both username and token must match an enabled server credential. Cleartext internal requests are rejected with `426` text `the internal API requires HTTPS`. A missing or invalid credential over HTTPS returns `401` with `WWW-Authenticate: Basic`.

- An **event** owns problems, the complete team roster, elapsed time, penalty, freeze boundary, and submissions (runs).
- A **contest** selects teams from that event through `codes` regexes and configures its scoreboard appearance. There is no automatic/default contest.
- A **site** selects a local team group and has a derived private revelation key. Set site filters to a subset of contest teams: secret-run filtering uses site regexes against the event's runs, and the server does not enforce the subset.
- Names identify resources, not separate display labels. Use nonempty URL-safe identifiers and percent-encode path segments. Event and contest body names must match their path identifiers. Use the matching site name too; the server accepts an empty site body name but normalizes it in storage.
- `codes` are Rust regular expressions, combined with OR. Matching is unanchored unless you add `^`/`$`. `[".*"]` matches every team; `[]` matches none. Team `login` values connect runs, filters, and media. `nome` is the displayed team name; `escola` is its institution.
- All timing fields use **seconds since event start**, not timestamps or minutes. `time_seconds < 0` keeps public contest details and runs closed; event and contest names remain discoverable. The server stores the last supplied time and **does not advance it automatically**. A controller or feeder must send updates.
- JSON requests are bare objects with `Content-Type: application/json`. Successful JSON responses have `data` and optionally `warnings`; errors have only `errors`, a list of `{code,message}`. Optional envelope fields are omitted, not null. Resource fields such as `salt`, `style`, and media templates may be null. `204` has no body. Metrics and WebSocket messages do not use the envelope. Error messages may be Portuguese; branch on HTTP status and `code`.
- Event state is **in memory**. Restarting the server loses API-created configuration and runs. Retain the source inputs to recreate them. Coordinate with any existing feeder so it does not overwrite manual changes.

## Worked setup: regional-2026 / brasil / fiemg

The following commands use a fictitious deployment. Replace the values with your server and credentials; use your deployment's trusted CA (`--cacert` if required).

```sh
export ANIMEITOR_URL='https://animeitor.example.com'
export ANIMEITOR_USER='operator'
export ANIMEITOR_TOKEN='replace-with-configured-token'
```

### 1. Inspect before creating

```sh
curl --fail-with-body -u "$ANIMEITOR_USER:$ANIMEITOR_TOKEN" "$ANIMEITOR_URL/internal/events"
```

A new server returns `200 {"data":[]}`. If `regional-2026` already exists, inspect it with `GET /internal/events/regional-2026`, its contests with `GET /internal/events/regional-2026/contests`, and each contest's sites with `GET /internal/events/regional-2026/contests/brasil/sites`. Internal contest/site lists contain full configurations, including salts, in unspecified order. Individual reads are also available at `GET /internal/contests/{event}/{contest}` and `GET /internal/sites/{event}/{contest}/{site}`.

Creation returns `409 conflict` for existing resources. Do not delete or replace an existing event just to retry setup: read it and decide whether it is the intended event. `PUT` is full replacement, not a merge or upsert; omitted optional fields reset to defaults.

### 2. Create the event before its children or runs

```sh
curl --fail-with-body -u "$ANIMEITOR_USER:$ANIMEITOR_TOKEN" \
  -H 'Content-Type: application/json' -X POST \
  "$ANIMEITOR_URL/internal/events/regional-2026" -d '{
    "name":"regional-2026",
    "problems":["A","B"],
    "teams":[{"login":"teambr001","escola":"Example University","nome":"Example Team"}],
    "score_freeze_time_seconds":14400,
    "penalty_seconds":1200,
    "time_seconds":-60
  }'
```

Expect `201` with the stored event under `data` (`salt` is null). The clock remains at -60 until updated. Omitted `time_seconds` defaults to **0**, which makes public contest data available immediately. Problem identifiers must match run `prob` values exactly.

### 3. Create a contest and a site

```sh
curl --fail-with-body -u "$ANIMEITOR_USER:$ANIMEITOR_TOKEN" \
  -H 'Content-Type: application/json' -X POST \
  "$ANIMEITOR_URL/internal/contests/regional-2026/brasil" -d '{
    "name":"brasil","codes":["^teambr"],
    "ouro":1,"prata":2,"bronze":3,
    "photo_url_format":"https://media.example.com/photos/{team_login}.webp",
    "sound_url_format":"https://media.example.com/sounds/{team_login}.mp3"
  }'

curl --fail-with-body -u "$ANIMEITOR_USER:$ANIMEITOR_TOKEN" \
  -H 'Content-Type: application/json' -X POST \
  "$ANIMEITOR_URL/internal/sites/regional-2026/brasil/fiemg" -d '{
    "name":"fiemg","codes":["^teambr001$"]
  }'
```

Each returns `201` with its configuration under `data`. Parents must already exist (`404` otherwise). `ouro`, `prata`, and `bronze` are inclusive 1-based medal thresholds, defaulting to 1, 2, and 3. `style` is an optional frontend style name. Media templates substitute `{team_login}`; absent templates default to `photos/{team_login}.webp` and `sounds/{team_login}.mp3` at the API origin. The example media domain is illustrative; replace it or omit those fields.

### 4. Verify configuration and obtain ready-to-use revelation URLs

```sh
curl --fail-with-body -u "$ANIMEITOR_USER:$ANIMEITOR_TOKEN" \
  "$ANIMEITOR_URL/internal/events/regional-2026/contests"
curl --fail-with-body -u "$ANIMEITOR_USER:$ANIMEITOR_TOKEN" \
  "$ANIMEITOR_URL/internal/events/regional-2026/contests/brasil/sites"
curl --fail-with-body -u "$ANIMEITOR_USER:$ANIMEITOR_TOKEN" \
  "$ANIMEITOR_URL/internal/events/regional-2026/revelation_urls"
```

The URL listing returns `200` with one entry per site across all event contests, sorted by contest and site:

```json
{"data":[{"contest":"brasil","site":"fiemg","url":"https://example.com/animeitor/regional-2026/brasil/?secret=EXAMPLE_KEY&sede=fiemg"}]}
```

The actual origin comes from the server's `public_url`, which may differ from the internal API origin. Any configured base path is replaced with `/animeitor/{event}/{contest}/`. The frontend must be served there. An existing event without sites returns `{"data":[]}`. This endpoint works before start and uses `Cache-Control: no-store`.

Open the returned URL for that site's revelation view. Its `sede` parameter selects the site; its `secret` parameter is also the Bearer token accepted by `runs_secret`. Use a URL parser to extract the parameter—there is no separate key retrieval endpoint. For the public scoreboard, remove the `secret` parameter; retain `sede` to select a site, or remove both parameters for the contest view. Treat revelation URLs as credentials; do not publish them alongside public links.

### 5. Ingest submissions and corrections

```sh
curl --fail-with-body -u "$ANIMEITOR_USER:$ANIMEITOR_TOKEN" \
  -H 'Content-Type: application/json' -X POST \
  "$ANIMEITOR_URL/internal/events/regional-2026/runs" -d '{
    "runs":[{"id":1,"team_login":"teambr001","prob":"A","time_seconds":56,"answer":"Y"}]
  }'
```

Expect `200 {"data":{"added":1,"updated":0}}`. `Y` means accepted, `N` incorrect, `?` pending/unknown, and `X` halted/unknown (the frontend treats it as unknown). This example may be preloaded before start; the server does not wait for the clock to reach the submission's timestamp before serving stored runs after start.

Resend ID 1 with changed fields to correct it (`updated:1`); an identical resend returns zero additions and updates. Unknown teams are ignored with `unknown_team` warnings alongside `data`. Fix the roster if those teams were intended to participate. Known-team runs with unknown problems reject the batch with `400 invalid_value`. Accepted entries are applied in `(time_seconds,id)` order; avoid multiple versions of one ID in a single request unless you intend that ordering.

### 6. Start and verify the public view

```sh
curl --fail-with-body -u "$ANIMEITOR_USER:$ANIMEITOR_TOKEN" \
  -H 'Content-Type: application/json' -X PATCH \
  "$ANIMEITOR_URL/internal/events/regional-2026/time" -d '{"time_seconds":60}'
```

Expect `200 {"data":{"time_seconds":60}}`. Continue PATCHing elapsed seconds from your controller or feeder. Public requests need no Basic credentials:

```sh
curl --fail-with-body "$ANIMEITOR_URL/api/events/regional-2026/contests"
curl --fail-with-body "$ANIMEITOR_URL/api/events/regional-2026/contests/brasil/contest"
curl --fail-with-body "$ANIMEITOR_URL/api/events/regional-2026/contests/brasil/config"
```

Expect `200` with `["brasil"]`, the selected team roster and timing, and presentation settings with site `fiemg`, respectively. If the public API uses a different origin, use that deployment URL. Before start the contest-name list still returns `200`, allowing landing-page links to countdowns; the state and configuration endpoints return `403 not_started`, which is expected.

Connect a WebSocket client to `/api/events/regional-2026/timer` (using `wss://` for HTTPS) for the immediate current timer and subsequent changes. Connect to `/api/events/regional-2026/contests/brasil/runs_ws` for replay and live submissions, one bare run object per text message. Runs at or after 14400 seconds are masked as `?` in this example. Repeated IDs replace earlier results. Reconnect and rebuild state after changing filters/freeze. Clearing stored runs does not clear stream replay history.

To verify private access, extract `secret` from the returned revelation URL, assign it to `ANIMEITOR_SITE_KEY`, and request:

```sh
curl --fail-with-body -H "Authorization: Bearer $ANIMEITOR_SITE_KEY" \
  "$ANIMEITOR_URL/api/events/regional-2026/contests/brasil/runs_secret"
```

Expect `200 {"data":{"runs":[...]}}` containing the site's actual answers. Before start, even a valid key returns `403 not_started`; missing/invalid keys after start return `403 invalid_key`.

## Maintenance and recovery

- Read before a full `PUT`. Event replacement preserves contests, sites, and runs; contest replacement preserves sites. Omitted optional values reset, including event time (0), medal defaults, and salts (null).
- `POST .../salt` at event, contest, or site scope sets the supplied salt, or generates one for an absent body/field, null, or empty string. Changing an event salt changes all its site keys; changing a contest salt changes its sites; changing a site salt affects only that site. Fetch revelation URLs again afterward. `PUT` with salt omitted/null clears that derivation input. Setting the same effective salt leaves keys unchanged.
- Salts are optional derivation inputs, not the revelation keys. The private deployment secret also participates; it is never returned by this API. Changing it changes all deployment keys.
- `DELETE` event removes all its children and runs; deleting a contest removes its sites but retains event runs. Deleting a site retains runs. `DELETE .../runs` clears stored submissions but retains WebSocket replay history and sends no reset message. Reconnecting may replay previously cleared submissions; recreate the event and its configuration when a clean stream history is required. Successful deletions return `204` without JSON.
- Handle `400` using the structured code (`invalid_json`, `missing_field`, `invalid_value`, `invalid_regex`), `401` by checking credentials, `404` by checking identifiers and parent creation, and `409` by inspecting the existing resource. Do not retry invalid payloads unchanged.

## Atomic incremental management

Use PATCH on `/internal/events/{event}`, `/internal/contests/{event}/{contest}`, or `/internal/sites/{event}/{contest}/{site}` to change selected fields. For example, `PATCH /internal/contests/regional-2026/brasil` with `{"ouro":4,"style":null}` changes only the gold threshold and clears the style. Omitted fields remain unchanged; explicit arrays replace whole lists. Null clears nullable fields (salt, style, media templates), but is rejected for required fields. Names cannot change; an unchanged name is accepted. Unknown fields and empty patches are rejected. Each operation reads, validates, and commits under a single store lock; invalid regexes or other validation failures leave all fields unchanged. Responses return the complete updated resource under `data`.

Item endpoints support incremental collections:

| Endpoint | Request and behavior |
| --- | --- |
| `POST /internal/events/{event}/teams` | `{login,escola,nome}` appends one team; `201` with the team |
| `GET /internal/events/{event}/teams/{login}` | `200` with one team |
| `PATCH /internal/events/{event}/teams/{login}` | Change `escola` and/or `nome`; login is immutable; `200` with the team |
| `DELETE /internal/events/{event}/teams/{login}` | Remove one team; `204` |
| `POST /internal/events/{event}/problems` | `{"problem":"C"}` appends a problem; `201` with the ordered list |
| `DELETE /internal/events/{event}/problems/{problem}` | Remove one unreferenced problem; `204` |
| `PATCH /internal/contests/{event}/{contest}/codes` | `{"add":["^teambr002$"],"remove":["^teambr001$"]}`; `200` with updated contest |
| `PATCH /internal/sites/{event}/{contest}/{site}/codes` | Same filter delta, returning the updated site |

Resource/item identifiers must be encoded as individual URL path segments. Team logins and problem identifiers must be nonempty for item creation. Duplicate creation returns `409 conflict`; missing resources return `404 not_found`. Legacy duplicate team logins or problem identifiers make item-level operations ambiguous and return `409`; repair the relevant array using full replacement. PATCH replacement arrays reject duplicate identifiers.

Removing a team or problem with stored runs returns `409 conflict`. To remove a team while intentionally retaining its submissions, use `DELETE .../teams/{login}?keep_runs=true`. Event PATCH replacing `teams` accepts the same query option. Retained runs and WebSocket history are unchanged and may still match regex streams. Recreating the login associates them with the team again; new ingestion while the team is absent is skipped with `unknown_team` warnings. **There is no override for removing a referenced problem.** Reordering problems is possible with a complete `problems` array in event PATCH. Existing full PUT behavior is unchanged and does not apply these new reference checks.

Filter deltas compare exact regex strings, preserve retained order, and append additions in request order. Adding an existing pattern or removing an absent pattern is a no-op; removal deletes all exact duplicates. A pattern cannot appear in both arrays. At least one addition or removal is required. The final regex set must compile before any changes are installed.

Atomic PATCH prevents lost updates to unrelated fields among incremental callers. It does not stop the existing feeder or another full PUT caller from subsequently replacing fields. Coordinate manual changes with source configuration. Reconnect run streams after changing contest filters or freeze time; these changes do not add a stream reset protocol.
