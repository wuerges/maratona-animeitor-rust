include .env

.PHONY: rebuild-client-for-release rebuild-server-for-release rebuild-docker-image run-standalone

run-debug-client:
	( cd client-v2 && trunk serve )

run-standalone-push:
	( cargo run -p cli \
		--bin simples -- \
		-p ${PUBLIC_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./client-v2/release: \
		--sedes ${SEDES}: \
		--secret ${SECRET} \
		-k api-key \
	)

run-standalone-loop:
	( cargo run -p cli \
		--bin simples -- \
		-p ${PUBLIC_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./client-v2/release: \
		--sedes ${SEDES}: \
		--secret ${SECRET} \
		-i ${BOCA_URL} \
	)

rebuild-client-for-release:
	@echo recompiling client...
	( cd client-v2 && trunk build --release -d release )

rebuild-server-for-release:
	@echo recompiling server...
	( cargo build -p cli --release --features vendored )

rebuild-docker-image:
	@echo rebuild docker image
	docker compose build

republish-docker-image: rebuild-docker-image
	docker compose push
