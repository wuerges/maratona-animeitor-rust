include .env

.PHONY: rebuild-client-for-release rebuild-server-for-release rebuild-docker-image run-standalone

run-debug-client:
	( cd client-v2 && URL_PREFIX="http://localhost:${PUBLIC_PORT}/api" trunk serve )

run-standalone: rebuild-client-for-release
	@echo recompiling client...
	( cargo run --manifest-path server/Cargo.toml \
		--bin simples -- \
		-p ${PUBLIC_PORT} \
		-v ./server/photos:photos \
		-v ./client-v2/release: \
		--sedes ${SEDES}: \
		--secret ${SECRET} ${BOCA_URL} \
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
