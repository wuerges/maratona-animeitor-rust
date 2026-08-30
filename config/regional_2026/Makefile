BOCA_URL ?=./tests/inputs/webcast-2023-1a-fase-final-prova.zip
PUBLIC_PORT ?= 80
TLS_PORT ?= 443
LOCAL_PORT ?= 8000
SERVER_RELEASE_DIR = target/release/

# PREFIX=http://localhost:8000
PREFIX=http://animeitor.naquadah.com.br
SERVER_URL=http://localhost:${PUBLIC_PORT}
INTERNAL_TOKEN=token-de-teste

HTTPS_OPTS ?= --tls-cert ./fullchain.pem \
            --tls-key ./privkey.pem \

# Contests, sites and salts are created via the internal API
# (POST /internal/contests/{event}/{name}); see doc/event-api.md.
#
# The client is built once by the root Makefile
# (make rebuild-client-for-release -> client-v2/release) and served from
# there; run `make install-config` after rebuilding it to apply this
# region's media prefixes.

.PHONY: install-config run-standalone run-standalone-loop run-local run-local-loop printurls-standalone rebuild-release-server

install-config:
	cp config/regional_2026/config.json client-v2/release/config.json

rebuild-release-server:
	cargo build -p server-v2 \
		--release

run-standalone:
	( ${SERVER_RELEASE_DIR}/simples \
		-p ${PUBLIC_PORT} \
		--tls-port ${TLS_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./client-v2/release: \
		-v ./client-v2/release:animeitor \
		${HTTPS_OPTS} \
		-t ${INTERNAL_TOKEN} \
	)

run-standalone-loop:
	( ${SERVER_RELEASE_DIR}/simples \
		-p ${PUBLIC_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./client-v2/release: \
		-v ./client-v2/release:animeitor \
		${HTTPS_OPTS} \
		-t ${INTERNAL_TOKEN} & ) ; \
	sleep 2 ; \
	cargo run -p cli --bin update_contest_state -- \
		-t ${INTERNAL_TOKEN} -i ${BOCA_URL} -s ${SERVER_URL}

printurls-standalone:
	( ${SERVER_RELEASE_DIR}/printurls \
		--server ${SERVER_URL} \
		--token ${INTERNAL_TOKEN} \
		--prefix ${PREFIX} \
	)

# Local testing: plain HTTP on a high port, no TLS. Uses cargo run so no
# release build is needed; override LOCAL_PORT if 8000 is taken.
run-local:
	cargo run -p server-v2 --bin simples -- \
		-p ${LOCAL_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./client-v2/release: \
		-v ./client-v2/release:animeitor \
		-t ${INTERNAL_TOKEN}

run-local-loop:
	( cargo run -p server-v2 --bin simples -- \
		-p ${LOCAL_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./client-v2/release: \
		-v ./client-v2/release:animeitor \
		-t ${INTERNAL_TOKEN} & ) ; \
	sleep 2 ; \
	cargo run -p cli --bin update_contest_state -- \
		-t ${INTERNAL_TOKEN} -i ${BOCA_URL} -s http://localhost:${LOCAL_PORT}
