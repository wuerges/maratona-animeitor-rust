include secret_env

BOCA_URL ?=./tests/inputs/pda-2024/pda-2024.zip
SECRET=./config/latam_2024_2025/Secrets_secret.toml
PUBLIC_PORT ?=80
SERVER_RELEASE_DIR = target/release/

# PREFIX=http://localhost:8000
PREFIX=http://animeitor.naquadah.com.br



SSH_ANIMEITOR=ew@animeitor.naquadah.com.br

.PHONY: run-debug-client rebuild-client run-standalone printurls-standalone rebuild-debug-server rebuild-release-server deploy

deploy:
	ssh ${SSH_ANIMEITOR} git -C maratona-animeitor-rust pull
	ssh ${SSH_ANIMEITOR} mkdir -p maratona-animeitor-rust/${SERVER_RELEASE_DIR}
	rsync -P ${SERVER_RELEASE_DIR}/animeitor-server ${SSH_ANIMEITOR}:maratona-animeitor-rust/${SERVER_RELEASE_DIR}/animeitor-server
	ssh ${SSH_ANIMEITOR} sudo setcap 'cap_net_bind_service=+ep' maratona-animeitor-rust/${SERVER_RELEASE_DIR}/animeitor-server
	rsync -P ${SERVER_RELEASE_DIR}/printurls ${SSH_ANIMEITOR}:maratona-animeitor-rust/${SERVER_RELEASE_DIR}/printurls
	rsync -P ${SECRET} ${SSH_ANIMEITOR}:maratona-animeitor-rust/${SECRET}

run-debug-client:
	( cd animeitor-client && \
		trunk serve )

rebuild-client:
	( cd animeitor-client && \
		trunk build --release -d naquadah )
	cp config/latam_2024_2025/config.json animeitor-client/naquadah/config.json

rebuild-debug-server:
	cargo build -p cli

rebuild-release-server:
	cargo build -p cli \
		--release \


SEDES_PARS=\
	--sedes ./config/latam_2024_2025/CCL.toml:ccl \
	--sedes ./config/latam_2024_2025/LatAm.toml:


run-standalone:
	( ${SERVER_RELEASE_DIR}/animeitor-server \
		-p ${PUBLIC_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./animeitor-client/naquadah: \
		${SEDES_PARS} \
		--secret ${SECRET} ${BOCA_URL} \
	)

printurls-standalone:
	( ${SERVER_RELEASE_DIR}/printurls \
		--prefix ${PREFIX} \
		${SEDES_PARS} \
		--secret ${SECRET} \
	)
