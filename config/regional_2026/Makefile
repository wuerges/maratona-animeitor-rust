include secret_env

BOCA_URL ?=./tests/inputs/webcast-2023-1a-fase-final-prova.zip
# SECRET=./config/regional_2026/Secrets.toml
SECRET=./config/regional_2026/Secrets_secret.toml
PUBLIC_PORT=80
SERVER_RELEASE_DIR = server/target/x86_64-unknown-linux-musl/release/

# PREFIX=http://localhost:8000
PREFIX=http://animeitor.naquadah.com.br
PHOTO_PREFIX=${PREFIX}/photos
SOUND_PREFIX=${PREFIX}/sounds

HTTPS_OPTS ?= --tls-cert ./fullchain.pem \
            --tls-key ./privkey.pem \

SEDES_OPTS= --sedes ./config/regional_2026/CCL.toml:ccl \
		--sedes ./config/regional_2026/CCL_Bolivia.toml:ccl-bolivia \
		--sedes ./config/regional_2026/CCL_Brasil.toml:ccl-brasil \
		--sedes ./config/regional_2026/CCL_Mexico.toml:ccl-mexico\
		--sedes ./config/regional_2026/CCL_Centro_America.toml:ccl-centro \
		--sedes ./config/regional_2026/LA.toml: \
		--sedes ./config/regional_2026/Brasil.toml:brasil \
		--sedes ./config/regional_2026/Bolivia.toml:bolivia \
		--sedes ./config/regional_2026/Centro_America.toml:centro \
		--sedes ./config/regional_2026/Mexico.toml:mexico \

SSH_ANIMEITOR=ew@animeitor.naquadah.com.br

.PHONY: run-debug-client rebuild-client run-standalone printurls-standalone rebuild-debug-server rebuild-release-server deploy

run-debug-client:
	( cd client-v2 && \
		PHOTO_PREFIX=${PHOTO_PREFIX} \
		PHOTO_URL_FORMAT=${PHOTO_URL_FORMAT} \
		SOUND_PREFIX=${SOUND_PREFIX} \
		SOUND_URL_FORMAT=${SOUND_URL_FORMAT} \
		URL_PREFIX=${PREFIX}/api \
		trunk serve )

rebuild-client:
	( cd client-v2 && \
		PHOTO_PREFIX=${PHOTO_PREFIX} \
		PHOTO_URL_FORMAT=${PHOTO_URL_FORMAT} \
		SOUND_PREFIX=${SOUND_PREFIX} \
		SOUND_URL_FORMAT=${SOUND_URL_FORMAT} \
		URL_PREFIX=${PREFIX}/api \
		trunk build --release -d naquadah )

rebuild-debug-server:
	cargo build --manifest-path server/Cargo.toml

rebuild-release-server:
	cargo build \
		--release \
		--target x86_64-unknown-linux-musl --features vendored \
		--manifest-path server/Cargo.toml
	sudo setcap 'cap_net_bind_service=+eip' ${SERVER_RELEASE_DIR}/simples
	
rebuild-release-server-dev:
	cargo build \
		--release \
		--features vendored \
		--manifest-path server/Cargo.toml

run-standalone-dev:
	( ./server/target/release/simples \
		-p ${PUBLIC_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./client-v2/naquadah: \
		${SEDES_OPTS} \
		--secret ${SECRET} -i ${BOCA_URL} \
	)

run-standalone:
	( ${SERVER_RELEASE_DIR}/simples \
		-p ${PUBLIC_PORT} \
		--tls-port 443 \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./client-v2/naquadah: \
		${SEDES_OPTS} \
		${HTTPS_OPTS} \
		--secret ${SECRET} -i ${BOCA_URL} \
	)
	
printurls-standalone:
	( ${SERVER_RELEASE_DIR}/printurls \
		--prefix ${PREFIX} \
		${SEDES_OPTS} \
		--secret ${SECRET} \
	)
