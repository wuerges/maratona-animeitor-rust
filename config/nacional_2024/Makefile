include secret_env

BOCA_URL ?=./tests/inputs/webcast-2023-1a-fase-final-prova.zip
# SECRET=./config/nacional_2024/Secrets.toml
SECRET=./config/nacional_2024/Secrets_secret.toml
PUBLIC_PORT=80
SERVER_RELEASE_DIR = target/x86_64-unknown-linux-musl/release/

# PREFIX=http://localhost:8000
PREFIX=http://animeitor.naquadah.com.br



SSH_ANIMEITOR=ew@animeitor.naquadah.com.br

.PHONY: run-debug-client rebuild-client run-standalone printurls-standalone rebuild-debug-server rebuild-release-server deploy

deploy:
	ssh ${SSH_ANIMEITOR} git -C maratona-animeitor-rust pull
	ssh ${SSH_ANIMEITOR} mkdir -p maratona-animeitor-rust/${SERVER_RELEASE_DIR}
	rsync -P ${SERVER_RELEASE_DIR}/simples ${SSH_ANIMEITOR}:maratona-animeitor-rust/${SERVER_RELEASE_DIR}/simples
	ssh ${SSH_ANIMEITOR} sudo setcap 'cap_net_bind_service=+ep' maratona-animeitor-rust/${SERVER_RELEASE_DIR}/simples
	rsync -P ${SERVER_RELEASE_DIR}/printurls ${SSH_ANIMEITOR}:maratona-animeitor-rust/${SERVER_RELEASE_DIR}/printurls
	rsync -P ${SECRET} ${SSH_ANIMEITOR}:maratona-animeitor-rust/${SECRET}

run-debug-client:
	( cd client-v2 && \
		trunk serve )

rebuild-client:
	( cd client-v2 && \
		trunk build --release -d naquadah )
	cp config/nacional_2024/config.json client-v2/naquadah/config.json

rebuild-debug-server:
	cargo build -p cli

rebuild-release-server:
	cargo build -p cli \
		--release \
		--target x86_64-unknown-linux-musl --features vendored \


SEDES_PARS=\
	--sedes ./config/nacional_2024/Brasil_CCL.toml:brasil-ccl \
	--sedes ./config/nacional_2024/Brasil.toml:brasil \
	--sedes ./config/nacional_2024/Caribe.toml:caribe \
	--sedes ./config/nacional_2024/CCL.toml:ccl \
	--sedes ./config/nacional_2024/Central.toml:central \
	--sedes ./config/nacional_2024/LatAm.toml: \
	--sedes ./config/nacional_2024/Mexico.toml:mexico \
	--sedes ./config/nacional_2024/NO.toml:north \
	--sedes ./config/nacional_2024/SO.toml:south


run-standalone:
	( ${SERVER_RELEASE_DIR}/simples \
		-p ${PUBLIC_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./client-v2/naquadah: \
		${SEDES_PARS} \
		--secret ${SECRET} ${BOCA_URL} \
	)

printurls-standalone:
	( ${SERVER_RELEASE_DIR}/printurls \
		--prefix ${PREFIX} \
		${SEDES_PARS} \
		--secret ${SECRET} \
	)
