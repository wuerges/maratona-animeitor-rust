# Consume an Animeitor event

The public API reads event scoreboards; configuration is performed through the authenticated internal API at `/internal/openapi.json` (`/internal/docs`). The public specification is at `/api/openapi.json` (`/api/docs`).

Discover event identifiers with `GET /api/events`, then contest identifiers with `GET /api/events/{event_name}/contests`. Fetch the contest's `/contest` for selected teams, problems, and timing and `/config` for appearance and site choices. Identifiers are nonempty path segments; percent-encode them when constructing URLs. Public JSON responses contain `data` on success or `errors:[{code,message}]` on failure. Optional envelope fields are absent. There are no salts or derived keys in public configurations.

All time values are seconds since event start, not timestamps. Before the supplied event time reaches zero, contest discovery, state, config, and submissions return `403 not_started`. Only event discovery, the event timer, and remote control remain available. The clock is supplied by a feeder or controller; the server does not advance it automatically.

Use WebSocket upgrades (`ws://` or `wss://`) for `/timer`, `/runs_ws`, and `/remote_control/{key}`. Successful upgrades return `101`; subsequent text frames carry bare JSON without envelopes. Their schemas are PublicTimer, Run, and ControlMessage respectively. Resource/pre-start handshake errors have no JSON body. A malformed WebSocket upgrade can also be rejected by the HTTP framework.

The timer sends its current state immediately and then changes, suppressing consecutive duplicates. The run stream replays history then sends new submissions and corrections. Replace prior values when an ID repeats. Runs at or after the event's freeze boundary have answer `?`, regardless of the actual answer. Filters and freeze boundary are captured at connection time; reconnect and rebuild state after changing them. Clearing stored runs does not clear stream replay history.

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
