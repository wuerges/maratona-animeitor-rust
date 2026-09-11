include .env
-include secret_env

.PHONY: run-basic run-config run-server run-feeder printurls generate-dev-certs \
	check-server check-feeder rebuild-client-for-release rebuild-server-for-release \
	rebuild-docker-image republish-docker-image run-debug-client help

# Default target: `make` prints the available commands.
help:
	@echo "Animeitor targets:"
	@echo "  make run-basic BOCA_URL=...                         Run the basic example"
	@echo "  make run-config CONFIG=... BOCA_URL=...             Run server and feeder"
	@echo "  make run-server                                      Run only the HTTPS server"
	@echo "  make run-feeder CONFIG=... BOCA_URL=...             Run feeder against a server"
	@echo "  make printurls CONFIG=...                            Print contest/reveal URLs"
	@echo "  make generate-dev-certs                              Generate local TLS certs"
	@echo "  make rebuild-client-for-release                      Build the release client"
	@echo "  make rebuild-server-for-release                      Build the release server"
	@echo "  make rebuild-docker-image                            Build the Docker image"
	@echo "  make republish-docker-image                          Build and push the image"
	@echo
	@echo "Defaults: CONFIG=${CONFIG}, SERVER_URL=${SERVER_URL}"

# Select an event with CONFIG. The server always uses HTTPS for internal API calls.
CONFIG ?= config/nacional_2026/event.toml
BASIC_CONFIG = config/basic/event.toml
PUBLIC_PORT ?= 8000
TLS_PORT ?= 8443
SERVER_URL ?= https://localhost:${TLS_PORT}
PREFIX ?= http://localhost:${PUBLIC_PORT}
TLS_CERT ?= config/dev-certs/localhost-cert.pem
TLS_KEY ?= config/dev-certs/localhost-key.pem
INTERNAL_TOKENS_FILE = internal_tokens.toml
INTERNAL_TOKEN_NAME ?= feeder-nacional-2026
INTERNAL_TOKEN ?=
BOCA_URL ?=
PHOTO_URL_FORMAT ?=
SOUND_URL_FORMAT ?=
SCORE_FREEZE_TIME_SECONDS ?=

VOLUMES = -v ./server/photos:photos -v ./server/sounds:sounds \
	-v ./animeitor-client/release: -v ./animeitor-client/release:animeitor
TLS_ARGS = --tls-cert "${TLS_CERT}" --tls-key "${TLS_KEY}" --tls-port ${TLS_PORT}
FEEDER_ARGS = $(if ${PHOTO_URL_FORMAT},--photo-url-format "${PHOTO_URL_FORMAT}") \
	$(if ${SOUND_URL_FORMAT},--sound-url-format "${SOUND_URL_FORMAT}") \
	$(if ${SCORE_FREEZE_TIME_SECONDS},--score-freeze-time-seconds ${SCORE_FREEZE_TIME_SECONDS})

check-server:
	@test -f "${INTERNAL_TOKENS_FILE}" || { echo "missing ${INTERNAL_TOKENS_FILE}; copy internal_tokens.toml.example" >&2; exit 2; }
	@test -f "${TLS_CERT}" || { echo "missing ${TLS_CERT}; run make generate-dev-certs" >&2; exit 2; }
	@test -f "${TLS_KEY}" || { echo "missing ${TLS_KEY}; run make generate-dev-certs" >&2; exit 2; }

check-feeder: check-server
	@test -n "${INTERNAL_TOKEN}" || { echo "INTERNAL_TOKEN is required" >&2; exit 2; }
	@test -n "${BOCA_URL}" || { echo "BOCA_URL is required" >&2; exit 2; }
	case "${SERVER_URL}" in https://*) ;; *) echo "SERVER_URL must use https://" >&2; exit 2 ;; esac

run-server: check-server
	cargo run -p server-v2 --bin animeitor-server -- \
		-p ${PUBLIC_PORT} ${VOLUMES} --internal-tokens "${INTERNAL_TOKENS_FILE}" ${TLS_ARGS}

run-config: check-feeder
	trap 'kill $$server_pid 2>/dev/null || true' EXIT INT TERM; \
	cargo run -p server-v2 --bin animeitor-server -- -p ${PUBLIC_PORT} \
		${VOLUMES} --internal-tokens "${INTERNAL_TOKENS_FILE}" ${TLS_ARGS} & \
	server_pid=$$!; sleep 2; \
	cargo run -p cli --bin animeitor-feeder -- \
		--internal-user "${INTERNAL_TOKEN_NAME}" --internal-token "${INTERNAL_TOKEN}" \
		--config "${CONFIG}" -i "${BOCA_URL}" -s "${SERVER_URL}" ${FEEDER_ARGS}

run-basic:
	$(MAKE) run-config CONFIG=${BASIC_CONFIG}

run-feeder: check-feeder
	cargo run -p cli --bin animeitor-feeder -- \
		--internal-user "${INTERNAL_TOKEN_NAME}" --internal-token "${INTERNAL_TOKEN}" \
		--config "${CONFIG}" -i "${BOCA_URL}" -s "${SERVER_URL}" ${FEEDER_ARGS}

printurls: check-server
	cargo run -p cli --bin printurls -- \
		--server "${SERVER_URL}" --user "${INTERNAL_TOKEN_NAME}" --token "${INTERNAL_TOKEN}" \
		--config "${CONFIG}" --prefix "${PREFIX}"

generate-dev-certs:
	mkdir -p config/dev-certs
	openssl req -x509 -newkey rsa:2048 -sha256 -nodes -days 825 \
		-keyout config/dev-certs/localhost-key.pem \
		-out config/dev-certs/localhost-cert.pem -subj '/CN=localhost' \
		-addext 'subjectAltName=DNS:localhost,IP:127.0.0.1,IP:::1'
	chmod 600 config/dev-certs/localhost-key.pem

run-debug-client:
	(cd animeitor-client && trunk serve)

rebuild-client-for-release:
	(cd animeitor-client && trunk build --release -d release --public-url /animeitor/)

rebuild-server-for-release:
	cargo build -p server-v2 --release

rebuild-docker-image:
	docker compose build

republish-docker-image: rebuild-docker-image
	docker compose push
