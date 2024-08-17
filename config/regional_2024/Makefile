BOCA_URL=./tests/inputs/webcast-2023-1a-fase-final-prova.zip
SECRET=./config/regional_2024/Secrets.toml
PUBLIC_PORT=8000
SERVER_RELEASE_DIR ?= ./server/target/release/

# PREFIX=http://localhost:8000
PREFIX=http://animeitor.naquadah.com.br

.PHONY: run-debug-client rebuid-client run-standalone printurls-standalone rebuild-debug-server rebuild-release-server

run-debug-client:
	( cd client-v2 && \
		PHOTO_PREFIX=${PHOTO_PREFIX} \
		SOUND_PREFIX=${SOUND_PREFIX} \
		URL_PREFIX=${PREFIX}/api \
		trunk serve )

rebuild-client:
	( cd client-v2 && \
		PHOTO_PREFIX=${PHOTO_PREFIX} \
		SOUND_PREFIX=${SOUND_PREFIX} \
		URL_PREFIX=${PREFIX}/api \
		trunk build --release -d naquadah )

rebuild-debug-server:
	cargo build --manifest-path server/Cargo.toml

rebuild-release-server:
	cargo build --release --manifest-path server/Cargo.toml
		
run-standalone:
	( ${SERVER_RELEASE_DIR}/simples \
		-p ${PUBLIC_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./client-v2/naquadah: \
		--sedes ./config/regional_2024/CCL.toml:ccl \
		--sedes ./config/regional_2024/LA.toml: \
		--sedes ./config/regional_2024/Brasil.toml:brasil \
		--sedes ./config/regional_2024/Bolivia.toml:bolivia \
		--sedes ./config/regional_2024/Centro_America.toml:centro \
		--sedes ./config/regional_2024/Chile.toml:chile \
		--sedes ./config/regional_2024/Mexico.toml:mexico \
		--secret ${SECRET} ${BOCA_URL} \
	)

printurls-standalone:
	( ${SERVER_RELEASE_DIR}/printurls \
		--prefix ${PREFIX} \
		--sedes ./config/regional_2024/CCL.toml:ccl \
		--sedes ./config/regional_2024/LA.toml: \
		--sedes ./config/regional_2024/Brasil.toml:brasil \
		--sedes ./config/regional_2024/Bolivia.toml:bolivia \
		--sedes ./config/regional_2024/Centro_America.toml:centro \
		--sedes ./config/regional_2024/Chile.toml:chile \
		--sedes ./config/regional_2024/Mexico.toml:mexico \
		--secret ${SECRET} \
	)