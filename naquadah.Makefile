all:
	@echo prog-americas-print-urls
	@echo prog-americas-run-server

PREFIX ?= http://animeitor.naquadah.com.br
BOCA_URL ?= ./tests/inputs/webcast_jones.zip

prog-americas-print-urls:
	RUST_LOG=info ./printurls --sedes ./config/americas.toml --secret ./config/americas_secret.toml --prefix ${PREFIX}

prog-americas-run-server:
	RUST_LOG=info ./simples --port 80  --sedes ./config/americas.toml --secret ./config/americas_secret.toml -v ./www/:  -v ./www-transparent/:webcast  ${BOCA_URL}