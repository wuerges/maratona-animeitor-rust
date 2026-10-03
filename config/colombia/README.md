# Colombia example

One event, `colombia`, with two contests:

- `Teams`: logins `team001`, `team002`, etc.
- `CCLs`: logins `ccl001`, `ccl002`, etc.

Each contest has one `General` site containing all its teams. Login filters are
case-sensitive and match the entire login: the prefix followed by one or more
digits. Other logins are excluded from both contests.

Set up `server.toml` and `event-secrets.toml` as described in the
[main README](../../README.md). Add the event's actual private webcast URL or
local ZIP path to the `[webcasts]` table in `event-secrets.toml`:

```toml
[webcasts]
colombia = "https://example.com/private/webcast.zip"
```

The webcast supplies the teams and runs for both contests. From the repository
root, run:

```bash
make run-config EVENT_CONFIG=config/colombia/event.toml
```

To print scoreboard and revelation URLs offline:

```bash
make printurls EVENT_CONFIG=config/colombia/event.toml
```
