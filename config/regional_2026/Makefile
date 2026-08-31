BOCA_URL ?= ./tests/inputs/1_fase_2026/webcast-final-depois-do-contest-moj.zip
PUBLIC_PORT ?= 80
TLS_PORT ?= 443
LOCAL_PORT ?= 8000
SERVER_RELEASE_DIR = target/release/

# PREFIX=http://localhost:8000
PREFIX=http://animeitor.naquadah.com.br
SERVER_URL=http://localhost:${PUBLIC_PORT}
INTERNAL_TOKEN=token-de-teste

HTTPS_OPTS ?= --tls-cert ./fullchain.pem \
            --tls-key ./privkey.pem \

# Media URLs, set by the feeder on every contest (the client substitutes
# {team_login} in the formats below).
PHOTO_URL_FORMAT = https://icpc-latam-br-fp-2026.moj.naquadah.com.br/api/v1/contest/team-photo?contest=icpc-latam-br-fp-2026&user={team_login}
SOUND_URL_FORMAT = https://icpc-latam-br-fp-2026.moj.naquadah.com.br/api/v1/contest/team-music?contest=icpc-latam-br-fp-2026&user={team_login}
MEDIA_OPTS = --photo-url-format '${PHOTO_URL_FORMAT}' --sound-url-format '${SOUND_URL_FORMAT}'

# Contests, sites and salts are created via the internal API
# (POST /internal/contests/{event}/{name}); see doc/event-api.md.
CONTEST_FILES = config/regional_2026/Brasil.toml \
                config/regional_2026/LA.toml \
                config/regional_2026/Mexico.toml \
                config/regional_2026/Centro_America.toml \
                config/regional_2026/Bolivia.toml \
                config/regional_2026/CCL.toml \
                config/regional_2026/CCL_Brasil.toml \
                config/regional_2026/CCL_Bolivia.toml \
                config/regional_2026/CCL_Mexico.toml \
                config/regional_2026/CCL_Centro_America.toml
SECRETS_FILE = config/regional_2026/Secrets_secret.toml
SECRETS_OPTS = $(if $(wildcard $(SECRETS_FILE)),--secrets $(SECRETS_FILE),)

#
# The client is built once by the root Makefile
# (make rebuild-client-for-release -> animeitor-client/release) and served
# from there; the media formats travel with the contest config, set by the
# feeder.

.PHONY: run-standalone run-standalone-loop run-local run-local-loop printurls-standalone rebuild-release-server

rebuild-release-server:
	cargo build -p server-v2 \
		--release

run-standalone:
	( ${SERVER_RELEASE_DIR}/animeitor-server \
		-p ${PUBLIC_PORT} \
		--tls-port ${TLS_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./animeitor-client/release: \
		-v ./animeitor-client/release:animeitor \
		${HTTPS_OPTS} \
		-t ${INTERNAL_TOKEN} \
	)

run-standalone-loop:
	( ${SERVER_RELEASE_DIR}/animeitor-server \
		-p ${PUBLIC_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./animeitor-client/release: \
		-v ./animeitor-client/release:animeitor \
		${HTTPS_OPTS} \
		-t ${INTERNAL_TOKEN} & ) ; \
	sleep 2 ; \
	cargo run -p cli --bin animeitor-feeder -- \
		-t ${INTERNAL_TOKEN} -i ${BOCA_URL} -s ${SERVER_URL} \
		$(addprefix -c ,$(CONTEST_FILES)) \
		${SECRETS_OPTS} \
		${MEDIA_OPTS}

printurls-standalone:
	( ${SERVER_RELEASE_DIR}/printurls \
		--server ${SERVER_URL} \
		--token ${INTERNAL_TOKEN} \
		--prefix ${PREFIX} \
	)

# Local testing: plain HTTP on a high port, no TLS. Uses cargo run so no
# release build is needed; override LOCAL_PORT if 8000 is taken.
run-local:
	cargo run -p server-v2 --bin animeitor-server -- \
		-p ${LOCAL_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./animeitor-client/release: \
		-v ./animeitor-client/release:animeitor \
		-t ${INTERNAL_TOKEN}

run-local-loop:
	( cargo run -p server-v2 --bin animeitor-server -- \
		-p ${LOCAL_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./animeitor-client/release: \
		-v ./animeitor-client/release:animeitor \
		-t ${INTERNAL_TOKEN} & ) ; \
	sleep 2 ; \
	cargo run -p cli --bin animeitor-feeder -- \
		-t ${INTERNAL_TOKEN} -i ${BOCA_URL} -s http://localhost:${LOCAL_PORT} \
		$(addprefix -c ,$(CONTEST_FILES)) \
		${SECRETS_OPTS} \
		${MEDIA_OPTS}
