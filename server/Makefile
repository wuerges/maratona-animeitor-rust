regenerate-openapi:
	rm openapi.json
	cargo run --bin printapi > openapi.json
	spectral lint openapi.json
