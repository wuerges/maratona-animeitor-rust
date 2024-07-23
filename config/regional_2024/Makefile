include .env

BOCA_URL=./tests/inputs/webcast-2023-1a-fase-final-prova.zip
SECRET=./config/regional_2024/Secrets.toml

.PHONY: rebuild-client-for-release rebuild-server-for-release rebuild-docker-image run-standalone

run-debug-client:
	( cd client-v2 && \
		PHOTO_PREFIX=http://localhost:8000/photos \
		SOUND_PREFIX=http://localhost:8000/sounds \
		URL_PREFIX=http://localhost:8000/api \
		trunk serve )

run-standalone:
	@echo recompiling client...
	( cargo run --manifest-path server/Cargo.toml \
		--bin simples -- \
		-p ${PUBLIC_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./client-v2/release: \
		--sedes ./config/regional_2024/CCL.toml:ccl \
		--sedes ./config/regional_2024/LA.toml: \
		--sedes ./config/regional_2024/Brasil.toml:brasil \
		--sedes ./config/regional_2024/Bolivia.toml:bolivia \
		--sedes ./config/regional_2024/Centro_America.toml:centro \
		--sedes ./config/regional_2024/Chile.toml:chile \
		--sedes ./config/regional_2024/Mexico.toml:mexico \
		--secret ${SECRET} ${BOCA_URL} \
	)

# --prefix http://animeitor.naquadah.com.br

printurls-standalone:
	@echo recompiling client...
	( cargo run --manifest-path server/Cargo.toml \
		--bin printurls -- \
		--prefix http://localhost:8000 \
		--sedes ./config/regional_2024/CCL.toml:ccl \
		--sedes ./config/regional_2024/LA.toml: \
		--sedes ./config/regional_2024/Brasil.toml:brasil \
		--sedes ./config/regional_2024/Bolivia.toml:bolivia \
		--sedes ./config/regional_2024/Centro_America.toml:centro \
		--sedes ./config/regional_2024/Chile.toml:chile \
		--sedes ./config/regional_2024/Mexico.toml:mexico \
		--secret ${SECRET} \
	)

rebuild-client-for-release:
	@echo recompiling client...
	( cd client-v2 && trunk build --release -d release )

rebuild-server-for-release:
	@echo recompiling server...
	( cd server && cargo build --release --target x86_64-unknown-linux-musl --features vendored )

rebuild-docker-image: rebuild-server-for-release rebuild-client-for-release
	@echo rebuild docker image
	docker compose build

republish-docker-image: rebuild-docker-image
	docker compose push
