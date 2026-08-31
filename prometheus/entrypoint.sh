#!/bin/sh
set -eu

# SERVER_URL accepts either a host:port or an HTTP(S) URL. Prometheus stores
# the scheme separately from the scrape target.
server_url="${SERVER_URL:-http://animeitor.naquadah.com.br}"
case "$server_url" in
  http://*)
    scheme=http
    target=${server_url#http://}
    ;;
  https://*)
    scheme=https
    target=${server_url#https://}
    ;;
  *)
    scheme=http
    target=$server_url
    ;;
esac
target=${target%%/*}

if [ -z "$target" ]; then
  echo "SERVER_URL must include a host" >&2
  exit 1
fi

# Escape the only characters with special meaning in sed replacement text.
escaped_target=$(printf '%s' "$target" | sed 's/[\\&|]/\\&/g')
sed \
  -e "s|__TARGET__|$escaped_target|g" \
  -e "s|__SCHEME__|$scheme|g" \
  /etc/prometheus/prometheus.yml.template > /prometheus/prometheus.yml

# password_file avoids exposing a token through the Prometheus config API.
printf '%s' "${INTERNAL_TOKEN:?INTERNAL_TOKEN must be set}" > /prometheus/internal-token
chmod 600 /prometheus/internal-token

exec /bin/prometheus \
  --config.file=/prometheus/prometheus.yml \
  --storage.tsdb.path=/prometheus
