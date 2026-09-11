# Animeitor server

See the [root README](../README.md) for the current Docker and host workflows.

The server is independent of event configuration:

```bash
cargo run -p server-v2 --bin animeitor-server -- --server-config server.toml
```

Run commands from the repository root. Copy a server configuration example and
generate the development certificates first. The feeder reads the public event
file and the private event-to-webcast mapping, then publishes state over HTTPS:

```bash
cargo run -p cli --bin animeitor-feeder -- \
  --server-config server.toml --event-config config/nacional_2026/event.toml \
  --event-secrets event-secrets.toml
cargo run -p cli --bin printurls -- \
  --server-config server.toml --event-config config/nacional_2026/event.toml
```

`printurls` works offline. Both it and server authentication derive keys using the
private server salt and the public event/contest/site values. Old keys without the
server salt are not accepted.

Historical Compose files for older images have been removed. Use the root
Makefile and generated Compose configuration for current deployments.
