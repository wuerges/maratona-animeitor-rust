# Protected server ingress

These examples optionally place nginx in front of Animeitor. Direct Axum serving
is supported; see [direct capacity testing](../capacity-testing.md). For these
proxy examples, use nginx as the sole public ingress. The server
binds HTTP and HTTPS to all interfaces. Application budgets cannot prevent raw
TCP, TLS-handshake, or network-bandwidth floods; use a firewall and, for volumetric
attacks, an upstream provider with filtering.

The Compose examples publish ports 8000 and 8443 on **127.0.0.1 only**. Host nginx
can reach those ports. A containerized proxy on the same Docker network should
use the `animeitor` service directly and omit host `ports` entirely. Do not
publish the upstream ports on `0.0.0.0`; that bypasses proxy protection. Native
systemd installations using this proxy topology must configure the host firewall
to allow server ports only from the proxy, covering IPv4 and IPv6. Allow public 443 at
the proxy. Keep internal administration on verified HTTPS.

## Install an example

1. Include `http-protection.conf` **once inside `http {}`**, before the selected
   server block. It defines shared zones, the upgrade header map, and a safe log
   format. It also bounds incomplete-header/body and idle HTTP connections.
2. Choose one server example. `actix-http-nginx-https.conf` (legacy filename) is a
   full HTTPS reverse proxy. Replace its public certificates, upstream CA path,
   and upstream TLS name. Its verified HTTPS upstream permits internal routes;
   forwarding internal credentials to the cleartext listener is rejected.
   `api-only-server.conf` serves static assets and proxies only `/api`, using
   Docker DNS. Its port 80 is for a private network behind a TLS-terminating edge;
   public-facing installations must configure TLS.
3. Run `nginx -t` with the complete installed configuration, then reload nginx.
   Check a public request, a WebSocket upgrade, and an authenticated HTTPS
   internal request. Verify that direct upstream access from outside fails.

Example enclosing configuration (adapt file paths):

```nginx
worker_processes auto;
worker_rlimit_nofile 16384;
events { worker_connections 8192; }
http {
    include /etc/nginx/mime.types;
    include /etc/nginx/animeitor/http-protection.conf;
    include /etc/nginx/animeitor/actix-http-nginx-https.conf;
}
```

The examples start with **20 requests/second per source**, burst 40; **64 active
requests/connections per source**, and **2,048 globally per server name**, returning
429 on exhaustion. nginx counts WebSockets against its connection budget. A
normal viewer uses two WebSockets (timer and runs); remote control is optional.
Shared NAT addresses require enough per-source capacity for their viewers.
Do not trust client-supplied forwarding headers for rate-limit identity. If a
load balancer precedes nginx, configure real-IP handling only for its known
addresses; otherwise all viewers appear to originate at that balancer.

Proxy connect/write/read deadlines are 5/10/90 seconds. Public timer/runs heartbeat pings every 30 seconds keep healthy connections open,
even with a frozen timer. Remote-control relays also send heartbeats, so healthy
idle controllers remain connected until credential expiry or revocation. The
client reconnects automatically after network failures; use backoff for custom clients.
Upgrades use the Connection header map; ordinary HTTP does not request an upgrade.

## Keep capabilities out of logs

Use `animeitor_safe` for access logs. It omits **all query strings** (the client
can carry `remote_control` there) and redacts session keys in both public and
internal remote-control paths. Do not add `$request`, `$request_uri`, or
Authorization headers to access logs. Request-level nginx errors may contain raw
URIs, so the server examples set error logging to `crit`. If lower-severity error
logs are needed for troubleshooting, first arrange URI redaction at the logging
collector and restrict retention/access. Apply the same rules at any external
load balancer, CDN, or log collector.

## Size and verify budgets

The application uses configurable process limits documented in the public API:
50,000 active WebSockets by default, 128 public and 32 internal requests, 32 detached
mutations, heartbeat liveness checks, and bounded messages/write deadlines.
Remote-control sessions have additional limits. Process budgets are configured in the server `[limits]` table; keep proxy limits within those budgets.

Compose demo examples limit memory to 2 GiB, disable swap, and cap CPU at two
cores. Native units require measured CPU/memory limits through a systemd drop-in;
they retain a 256-task budget. These are example budgets, not capacity guarantees: load
the real roster, run replay, and static assets before deciding whether they fit.
If MemoryMax is reached, the server can be killed; memory-backed events disappear.
Docker server logs rotate at 10 MiB with three files. Configure journald retention
for native services as part of host management.

Keep file-descriptor limits finite and above expected sockets plus storage and
runtime overhead. nginx needs descriptors for both client and upstream sockets.
Raising `nofile` alone does not prevent DoS.

Before rollout, measure RSS, CPU, open descriptors, response latency, and live
connection count during an expected viewer/reconnect burst. Exercise unknown
keys, session expiry/revocation, slow readers, and excess requests locally. Check
that 429/408 responses recover after pressure stops and that feeder writes still
progress. The repository changes do not deploy or validate your production
firewall/proxy configuration.
