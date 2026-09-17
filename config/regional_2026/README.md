# regional_2026

The committed event.toml contains this event's complete contest/site definitions
and public key-derivation values. No external contest files are loaded.

Copy the root server configuration and event-secrets examples as described in
[the main README](../../README.md). The private webcast mapping must contain
this event's name. Then run:

```bash
# Use the host server configuration example:
make run-config EVENT_CONFIG=config/regional_2026/event.toml
make printurls EVENT_CONFIG=config/regional_2026/event.toml
```

The server itself needs only server.toml and learns events through the internal API.

The root Compose demo runs Jones. To use this event in Docker, update the event mounts and arguments in Compose and provide its actual webcast in the private source mapping.
