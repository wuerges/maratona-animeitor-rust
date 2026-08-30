BOCA_URL ?=./tests/inputs/webcast-2023-1a-fase-final-prova.zip
PUBLIC_PORT=80
SERVER_RELEASE_DIR = target/release/

# PREFIX=http://localhost:8000
PREFIX=http://animeitor.naquadah.com.br
SERVER_URL=http://localhost:${PUBLIC_PORT}
INTERNAL_TOKEN=token-de-teste

HTTPS_OPTS ?= --tls-cert ./fullchain.pem \
            --tls-key ./privkey.pem \

# Contests, sites and salts are created via the internal API
# (POST /internal/contests/{event}/{name}); see doc/event-api.md.

.PHONY: run-debug-client rebuild-client run-standalone run-standalone-loop printurls-standalone rebuild-debug-server rebuild-release-server

run-debug-client:
	( cd client-v2 && \
		trunk serve )

rebuild-client:
	( cd client-v2 && \
		trunk build --release -d naquadah )
	cp config/regional_2026/config.json client-v2/naquadah/config.json

rebuild-debug-server:
	cargo build -p server-v2

rebuild-release-server:
	cargo build -p server-v2 \
		--release
	sudo setcap 'cap_net_bind_service=+eip' ${SERVER_RELEASE_DIR}/simples

run-standalone:
	( ${SERVER_RELEASE_DIR}/simples \
		-p ${PUBLIC_PORT} \
		--tls-port 443 \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./client-v2/naquadah: \
		-v ./client-v2/naquadah:animeitor \
		${HTTPS_OPTS} \
		-t ${INTERNAL_TOKEN} \
	)

run-standalone-loop:
	( ${SERVER_RELEASE_DIR}/simples \
		-p ${PUBLIC_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./client-v2/naquadah: \
		-v ./client-v2/naquadah:animeitor \
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
