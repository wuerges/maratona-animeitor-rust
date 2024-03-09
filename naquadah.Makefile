prog-americas-print-urls:
	RUST_LOG=info ./printurls --sedes ./config/americas.toml --secret ./config/americas_secret.toml --prefix http://localhost:8000

prog-americas-run-server:
	RUST_LOG=info ./simples --sedes ./config/americas.toml --secret ./config/americas_secret.toml -v ./www/:  -v ./www-transparent/:  ${BOCA_URL}
