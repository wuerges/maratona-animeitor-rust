.DEFAULT_GOAL := help
EVENT_CONFIG ?= config/jones/event.toml
EVENT_SECRETS ?= event-secrets.toml
SERVER_CONFIG ?= server.toml
IMAGE = wuerges/animeitor:latest
CONFIG_ARGS = --event-config "${EVENT_CONFIG}" --server-config "${SERVER_CONFIG}"
FEEDER_ARGS = ${CONFIG_ARGS} --event-secrets "${EVENT_SECRETS}"

.PHONY: help run-basic run-config run-server run-feeder printurls generate-dev-certs \
 rebuild-client-for-release rebuild-server-for-release rebuild-docker-image \
 republish-docker-image run-debug-client run-docker stop-docker monitor-docker
help:
	@echo 'make rebuild-docker-image    Build image (no configuration required)'
	@echo 'make run-docker              Start the example Compose stack'
	@echo 'make stop-docker             Stop the example stack'
	@echo 'make monitor-docker          Start stack with Prometheus on port 9090'
	@echo 'make run-config              Run server and feeder without Docker'
	@echo 'make run-server / run-feeder  Run one program without Docker'
	@echo 'make printurls               Print URLs offline without Docker'
	@echo 'Host selectors: EVENT_CONFIG, EVENT_SECRETS, SERVER_CONFIG'

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
	$(MAKE) run-config EVENT_CONFIG=config/jones/event.toml

generate-dev-certs:
	mkdir -p config/dev-certs
	openssl req -x509 -newkey rsa:2048 -sha256 -nodes -days 825 \
		-keyout config/dev-certs/localhost-key.pem \
		-out config/dev-certs/localhost-cert.pem -subj '/CN=localhost' \
		-addext 'subjectAltName=DNS:localhost,DNS:animeitor,IP:127.0.0.1,IP:::1'
	chmod 600 config/dev-certs/localhost-key.pem

run-docker:
	docker compose up
monitor-docker:
	docker compose --profile monitoring up -d
stop-docker:
	docker compose --profile monitoring down
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
