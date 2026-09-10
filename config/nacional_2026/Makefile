# Configuration for the 2026 second phase.  The contest/site split was copied
# from nacional_2025 as a starting point; confirm team-login prefixes against
# the 2026 webcast before production use.

# Local deployment values and credentials belong here, not in this Makefile.
-include config/nacional_2026/secret_env

# Set this to the 2026 webcast URL (or a downloaded zip) in secret_env or on
# the command line: make -f config/nacional_2026/Makefile run-feeder BOCA_URL=...
BOCA_URL ?=
EVENT ?= nacional-2026
PUBLIC_PORT ?= 8000
TLS_PORT ?= 443
LOCAL_PORT ?= 8000
SERVER_RELEASE_DIR = target/release/

PREFIX ?= http://animeitor.naquadah.com.br
SERVER_URL ?= http://localhost:${PUBLIC_PORT}
INTERNAL_TOKEN ?= token-de-teste

# Set these in secret_env for the production server. Keeping TLS opt-in lets
# `run-standalone-loop` work out of the box for a local rehearsal.
HTTPS_OPTS ?=

# Optional formats are published in each contest config. The client replaces
# {team_login}; leave either variable empty when the static client defaults
# should be used instead.
PHOTO_URL_FORMAT ?=
SOUND_URL_FORMAT ?=
MEDIA_OPTS = $(if $(PHOTO_URL_FORMAT),--photo-url-format '$(PHOTO_URL_FORMAT)') $(if $(SOUND_URL_FORMAT),--sound-url-format '$(SOUND_URL_FORMAT)')

# Optional freeze override, in seconds.
FREEZE_OPTS = $(if $(SCORE_FREEZE_TIME_SECONDS),--score-freeze-time-seconds $(SCORE_FREEZE_TIME_SECONDS))

# This is the set used by nacional_2025. Mexico_CCL and SO_CCL remain in this
# directory as reference until their inclusion is confirmed.
CONTEST_FILES = config/nacional_2026/Brasil.toml \
                config/nacional_2026/Brasil_CCL.toml \
                config/nacional_2026/Caribe.toml \
                config/nacional_2026/CCL.toml \
                config/nacional_2026/Central.toml \
                config/nacional_2026/LatAm.toml \
                config/nacional_2026/Mexico.toml \
                config/nacional_2026/NO.toml \
                config/nacional_2026/SO.toml
SECRETS_FILE = config/nacional_2026/Secrets_secret.toml
SECRETS_OPTS = $(if $(wildcard $(SECRETS_FILE)),--secrets $(SECRETS_FILE))

.PHONY: rebuild-release-server run-standalone run-standalone-loop run-feeder run-local run-local-loop printurls-standalone

rebuild-release-server:
	cargo build -p server-v2 --release

run-standalone:
	${SERVER_RELEASE_DIR}/animeitor-server \
		-p ${PUBLIC_PORT} \
		--tls-port ${TLS_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./animeitor-client/release: \
		-v ./animeitor-client/release:animeitor \
		${HTTPS_OPTS} \
		-t ${INTERNAL_TOKEN}

run-standalone-loop:
	@test -n "${BOCA_URL}" || { echo "BOCA_URL must point to the 2026 webcast" >&2; exit 2; }; \
	cargo run -p server-v2 --bin animeitor-server -- \
		-p ${PUBLIC_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./animeitor-client/release: \
		-v ./animeitor-client/release:animeitor \
		${HTTPS_OPTS} \
		-t ${INTERNAL_TOKEN} & server_pid=$$!; \
	trap 'kill $$server_pid 2>/dev/null; wait $$server_pid 2>/dev/null' EXIT INT TERM; \
	sleep 2; \
	$(MAKE) -f config/nacional_2026/Makefile run-feeder SERVER_URL=${SERVER_URL}

run-feeder:
	@test -n "${BOCA_URL}" || (echo "BOCA_URL must point to the 2026 webcast" >&2; exit 2)
	cargo run -p cli --bin animeitor-feeder -- \
		-t ${INTERNAL_TOKEN} -i ${BOCA_URL} -s ${SERVER_URL} --event ${EVENT} \
		$(addprefix -c ,$(CONTEST_FILES)) \
		${SECRETS_OPTS} \
		${MEDIA_OPTS} \
		${FREEZE_OPTS}

printurls-standalone:
	cargo run -p cli --bin printurls -- \
		--server ${SERVER_URL} \
		--token ${INTERNAL_TOKEN} \
		--event ${EVENT} \
		--prefix ${PREFIX}

run-local:
	cargo run -p server-v2 --bin animeitor-server -- \
		-p ${LOCAL_PORT} \
		-v ./server/photos:photos \
		-v ./server/sounds:sounds \
		-v ./animeitor-client/release: \
		-v ./animeitor-client/release:animeitor \
		-t ${INTERNAL_TOKEN}

run-local-loop:
	$(MAKE) -f config/nacional_2026/Makefile run-standalone-loop \
		PUBLIC_PORT=${LOCAL_PORT} \
		SERVER_URL=http://localhost:${LOCAL_PORT} \
		HTTPS_OPTS=
