include .env

BOCA_URL=./tests/inputs/webcast-2023-1a-fase-final-prova.zip
SECRET=./config/regional_2024/Secrets.toml
PREFIX=http://animeitor.naquadah.com.br
# PREFIX=http://localhost:8000

PHOTO_PREFIX ?= ${PREFIX}/photos
SOUND_PREFIX ?= ${PREFIX}/sounds

.PHONY: run-debug-client rebuid-client run-standalone printurls-standalone

run-debug-client:
	( cd client-v2 && \
		PHOTO_PREFIX=${PHOTO_PREFIX} \
		SOUND_PREFIX=${SOUND_PREFIX} \
		URL_PREFIX=${PREFIX}/api \
		trunk serve )

rebuid-client:
	( cd client-v2 && \
		PHOTO_PREFIX=${PHOTO_PREFIX} \
		SOUND_PREFIX=${SOUND_PREFIX} \
		URL_PREFIX=${PREFIX}/api \
		trunk build --release -d naquadah )

run-standalone:
	( cargo run --manifest-path server/Cargo.toml \
		--bin simples -- \
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
	( cargo run --manifest-path server/Cargo.toml \
		--bin printurls -- \
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
