.DEFAULT_GOAL := help
EVENT_CONFIG ?= config/nacional_2026/event.toml
EVENT_SECRETS ?= event-secrets.toml
SERVER_CONFIG ?= server.toml
IMAGE = wuerges/animeitor:latest
GENERATED = ${CURDIR}/.generated
CONFIG_ARGS = --event-config "${EVENT_CONFIG}" --server-config "${SERVER_CONFIG}"
FEEDER_ARGS = ${CONFIG_ARGS} --event-secrets "${EVENT_SECRETS}"

.PHONY: help run-basic run-config run-server run-feeder printurls generate-dev-certs \
 rebuild-client-for-release rebuild-server-for-release rebuild-docker-image \
 republish-docker-image run-debug-client configure-docker run-docker stop-docker monitor-docker
help:
	@echo 'make rebuild-docker-image    Build image (no configuration required)'
	@echo 'make run-docker              Generate Compose configuration and start'
	@echo 'make stop-docker             Stop the generated stack'
	@echo 'make monitor-docker          Start stack with Prometheus on port 9090'
	@echo 'make run-config              Run server and feeder without Docker'
	@echo 'make run-server / run-feeder  Run one program without Docker'
	@echo 'make printurls               Print URLs offline without Docker'
	@echo 'Selectors: EVENT_CONFIG, EVENT_SECRETS, SERVER_CONFIG'

run-server:
	cargo run -p server-v2 --bin animeitor-server -- --server-config "${SERVER_CONFIG}"
run-feeder:
	cargo run -p cli --bin animeitor-feeder -- ${FEEDER_ARGS}
printurls:
	cargo run -p cli --bin printurls -- ${CONFIG_ARGS}
run-config:
	cargo build -p server-v2 -p cli
	trap 'kill $$server_pid 2>/dev/null || true' EXIT INT TERM; \
	target/debug/animeitor-server --server-config "${SERVER_CONFIG}" & \
	server_pid=$$!; \
	target/debug/animeitor-feeder ${FEEDER_ARGS}
run-basic:
	$(MAKE) run-config EVENT_CONFIG=config/basic/event.toml

generate-dev-certs:
	mkdir -p config/dev-certs
	openssl req -x509 -newkey rsa:2048 -sha256 -nodes -days 825 \
		-keyout config/dev-certs/localhost-key.pem \
		-out config/dev-certs/localhost-cert.pem -subj '/CN=localhost' \
		-addext 'subjectAltName=DNS:localhost,DNS:animeitor,IP:127.0.0.1,IP:::1'
	chmod 600 config/dev-certs/localhost-key.pem

# Mount the configuration roots at their host paths. The helper needs no socket.
configure-docker:
	mkdir -p "${GENERATED}"
	docker run --rm --user "$$(id -u):$$(id -g)" \
		--mount 'type=bind,source=${CURDIR},target=${CURDIR},readonly' \
		--mount 'type=bind,source=$(abspath ${EVENT_CONFIG}),target=$(abspath ${EVENT_CONFIG}),readonly' \
		--mount 'type=bind,source=$(abspath ${EVENT_SECRETS}),target=$(abspath ${EVENT_SECRETS}),readonly' \
		--mount 'type=bind,source=$(abspath ${SERVER_CONFIG}),target=$(abspath ${SERVER_CONFIG}),readonly' \
		--mount 'type=bind,source=${GENERATED},target=${GENERATED}' \
		--workdir "${CURDIR}" --entrypoint /animeitor-config ${IMAGE} \
		compose ${FEEDER_ARGS} --output-dir "${GENERATED}"
run-docker: configure-docker
	docker compose -f "${GENERATED}/compose.json" up
monitor-docker: configure-docker
	docker compose -f "${GENERATED}/compose.json" --profile monitoring up -d
stop-docker:
	docker compose -f "${GENERATED}/compose.json" down
run-debug-client:
	(cd animeitor-client && trunk serve)
rebuild-client-for-release:
	(cd animeitor-client && trunk build --release -d release --public-url /animeitor/)
rebuild-server-for-release:
	cargo build -p server-v2 -p cli --release
rebuild-docker-image:
	docker build -t ${IMAGE} .
republish-docker-image: rebuild-docker-image
	docker push ${IMAGE}
