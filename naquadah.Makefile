all:
	@echo prog-americas-print-urls
	@echo prog-americas-run-server

PREFIX ?= http://animeitor.naquadah.com.br
# BOCA_URL ?= ./tests/inputs/webcast-2023-1a-fase-final-prova.zip
BOCA_URL ?= https://global.naquadah.com.br/limaojapones/admin/report/webcast.php?webcastcode=geral33876

prog-americas-print-urls:
	RUST_LOG=info ./printurls --sedes ./config/americas.toml --secret ./config/americas_secret.toml --prefix ${PREFIX}

prog-americas-run-server:
	RUST_LOG=info ./simples --port 80  --sedes ./config/americas.toml --secret ./config/americas_secret.toml -v ./www/:  -v ./www-transparent/:webcast  -v ./www-chroma/:chroma  ${BOCA_URL}

enable-server-port-80:
	sudo setcap 'cap_net_bind_service=+ep' ./simples
