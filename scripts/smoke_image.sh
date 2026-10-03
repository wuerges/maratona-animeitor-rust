#!/usr/bin/env bash
set -euo pipefail
image=${1:?usage: smoke_image.sh IMAGE}
work=$(mktemp -d)
container=""
cleanup() {
  if [[ -n "$container" ]]; then docker logs "$container"; docker rm -f "$container" >/dev/null; fi
  rm -rf "$work"
}
trap cleanup EXIT
openssl req -x509 -newkey rsa:2048 -nodes -days 1 -keyout "$work/key.pem" -out "$work/cert.pem" -subj /CN=localhost >/dev/null 2>&1
cat > "$work/server.toml" <<'TOML'
revelation_salt = "ci-only-salt"
public_port = 8000
tls_port = 8443
tls_cert = "/smoke/cert.pem"
tls_key = "/smoke/key.pem"
tls_ca_cert = "/smoke/cert.pem"
server_url = "https://localhost:8443"
public_url = "http://localhost:8000"
client_token = "ci"
[[tokens]]
name = "ci"
token = "ci-only-token"
role = "read-write"
events = [".*"]
[[assets]]
directory = "/dist"
path = ""
[[assets]]
directory = "/dist"
path = "animeitor"
TOML
container=$(docker run -d -p 127.0.0.1::8000 -p 127.0.0.1::8443 -v "$work:/smoke:ro" "$image" --server-config /smoke/server.toml)
port=$(docker port "$container" 8000/tcp | cut -d: -f2)
tls_port=$(docker port "$container" 8443/tcp | cut -d: -f2)
for attempt in {1..30}; do
  if curl -fsS "http://127.0.0.1:$port/" -o "$work/index.html"; then break; fi
  sleep 1
done
test -s "$work/index.html"
curl -fkSs -u ci:ci-only-token "https://127.0.0.1:$tls_port/internal/capabilities" > "$work/capabilities.json"
python3 - "$work" <<'PY'
import json,pathlib,sys
root=pathlib.Path(sys.argv[1])
assert 'wasm' in root.joinpath('index.html').read_text(), 'missing WebAssembly frontend'
assert 'read-write' in root.joinpath('capabilities.json').read_text(), 'internal authentication failed'
json.loads(root.joinpath('capabilities.json').read_text())
PY
for binary in animeitor-admin animeitor-feeder printurls; do
  docker run --rm --entrypoint "/$binary" "$image" --help >/dev/null
done
