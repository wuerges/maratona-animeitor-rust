all:
	@echo prog-americas-print-urls
	@echo prog-americas-run-server
	@echo prog-americas-run-feeder

PREFIX ?= http://animeitor.naquadah.com.br
# BOCA_URL ?= ./tests/inputs/webcast-2023-1a-fase-final-prova.zip
BOCA_URL ?= https://global.naquadah.com.br/limaojapones/admin/report/webcast.php?webcastcode=geral33876
SERVER_URL ?= http://localhost:8000
INTERNAL_TOKEN ?= token-de-teste

# Contests, sites and salts are created via the internal API (`POST /internal/contests/{event}/{name}`).

prog-americas-print-urls:
	RUST_LOG=info ./printurls --server ${SERVER_URL} --token ${INTERNAL_TOKEN} --prefix ${PREFIX}

prog-americas-run-server:
	RUST_LOG=info ./simples --port 80 -v ./www/: -v ./www/:animeitor -v ./www-transparent/:webcast -v ./www-chroma/:chroma -t ${INTERNAL_TOKEN}

prog-americas-run-feeder:
	RUST_LOG=info ./update_contest_state -t ${INTERNAL_TOKEN} -i ${BOCA_URL} -s ${SERVER_URL} --event default

enable-server-port-80:
	sudo setcap 'cap_net_bind_service=+ep' ./simples
