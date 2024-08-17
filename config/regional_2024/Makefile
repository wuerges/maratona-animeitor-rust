BOCA_URL=./tests/inputs/webcast-2023-1a-fase-final-prova.zip
# SECRET=./config/regional_2024/Secrets.toml
SECRET=./config/regional_2024/Secrets_secret.toml
PUBLIC_PORT=80
SERVER_RELEASE_DIR = server/target/x86_64-unknown-linux-musl/release/

# PREFIX=http://localhost:8000
PREFIX=http://animeitor.naquadah.com.br
PHOTO_PREFIX=${PREFIX}/photos
SOUND_PREFIX=${PREFIX}/sounds

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
	cargo build \
		--release \
		--target x86_64-unknown-linux-musl --features vendored \
		--manifest-path server/Cargo.toml
		
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

