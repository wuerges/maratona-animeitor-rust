include .env

.PHONY: rebuild-client-for-release rebuild-server-for-release rebuild-docker-image run-standalone

run-debug-client:
	( cd client-v2 && trunk serve )

run-standalone-push:
	( cargo run -p server-v2 \
		--bin simples -- \
		-p ${PUBLIC_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./client-v2/release: \
		-v ./client-v2/release:animeitor \
		-t ${INTERNAL_TOKEN} \
	)

# Runs simples and the BOCA feeder (publishes via the internal API).
run-standalone-loop:
	( cargo run -p server-v2 --bin simples -- -p ${PUBLIC_PORT} \
		-v ./server/photos:photos -v ./server/sounds:sounds \
		-v ./client-v2/release: -v ./client-v2/release:animeitor \
		-t ${INTERNAL_TOKEN} & ) ; \
	sleep 2 ; \
	cargo run -p cli --bin update_contest_state -- \
		-t ${INTERNAL_TOKEN} -i ${BOCA_URL} -s http://localhost:${PUBLIC_PORT}

rebuild-client-for-release:
	@echo recompiling client...
	( cd client-v2 && trunk build --release -d release --public-url /animeitor/ )

rebuild-server-for-release:
	@echo recompiling server...
	( cargo build -p server-v2 --release )

rebuild-docker-image:
	@echo rebuild docker image
	docker compose build

republish-docker-image: rebuild-docker-image
	docker compose push
