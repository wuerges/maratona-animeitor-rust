all:
	@echo build-server
	@echo build-client
	@echo build-client-ccl
	@echo run-standalone

build-server:
	docker build \
		-t wuerges/animeitor server
	docker tag wuerges/animeitor wuerges/animeitor:0.12.0
	docker tag wuerges/animeitor wuerges/animeitor:latest
	docker push wuerges/animeitor:0.12.0
	docker push wuerges/animeitor:latest

build-client:
	docker build -f client.Dockerfile \
		--build-arg "REMOVE_CCL=1" \
		--build-arg "URL_PREFIX=http://animeitor.naquadah.com.br:8000" \
		--build-arg "PHOTO_PREFIX=https://photos.naquadah.com.br/photos" \
		-t wuerges/animeitor-client .
	docker tag wuerges/animeitor-client wuerges/animeitor-client:0.12.0
	docker tag wuerges/animeitor-client wuerges/animeitor-client:latest
	docker push wuerges/animeitor-client:0.12.0
	docker push wuerges/animeitor-client:latest

build-client-ccl:
	docker build -f client.Dockerfile \
		--build-arg "URL_PREFIX=http://animeitor.naquadah.com.br:8001" \
		--build-arg "PHOTO_PREFIX=https://photos.naquadah.com.br/photos" \
		-t wuerges/animeitor-client-ccl .
	docker tag wuerges/animeitor-client-ccl wuerges/animeitor-client-ccl:0.12.0
	docker tag wuerges/animeitor-client-ccl wuerges/animeitor-client-ccl:latest
	docker push wuerges/animeitor-client-ccl:0.12.0
	docker push wuerges/animeitor-client-ccl:latest

BOCA_URL ?= ../tests/inputs/webcast_jones.zip

run-standalone:
	@echo recompiling client...
	( cd client && REMOVE_CCL=0 wasm-pack build . --release --out-dir www/pkg --target web --out-name package )
	@echo running server...
	( cd server && RUST_LOG=info cargo run --bin simples -- -v ../client/www: --sedes ../config/basic.toml --secret ../config/basic_secret.toml  ${BOCA_URL} )

prog-americas-build-client:
	@echo recompiling client...
	( cd client && REMOVE_CCL=0 PHOTO_PREFIX=https://photos.naquadah.com.br/photos wasm-pack build . --release --out-dir www/pkg --target web --out-name package )

prog-americas-sync-client:
	rsync -av client/www/ ew@animeitor.naquadah.com.br:www
	rsync -av client/www/ ew@animeitor.naquadah.com.br:www-transparent
	rsync -v client/www/static/styles-transparent.css ew@animeitor.naquadah.com.br:www-transparent/static/styles-transparent.css

prog-americas-debug-server:
	@echo running server...
	( cd server && RUST_LOG=info cargo run --bin simples -- -v ../client/www: --sedes ../config/americas.toml --secret ../config/americas_secret.toml  ${BOCA_URL} )


prog-americas-build-server:
	@echo building the server
	( cd server && cargo build --release --target x86_64-unknown-linux-musl )

prog-americas-sync-server:
	rsync -v server/target/x86_64-unknown-linux-musl/release/simples ew@animeitor.naquadah.com.br:simples
	rsync -v server/target/x86_64-unknown-linux-musl/release/printurls ew@animeitor.naquadah.com.br:printurls

prog-americas-sync-configs:
	rsync -av config/ ew@animeitor.naquadah.com.br:config
	rsync -av tests/ ew@animeitor.naquadah.com.br:tests
	rsync -v naquadah.Makefile ew@animeitor.naquadah.com.br:Makefile

prog-americas-print-urls:
	RUST_LOG=info ./server/target/release/printurls --sedes ./config/americas.toml --secret ./config/americas_secret.toml --prefix http://localhost:8000

prog-americas-run-server:
	RUST_LOG=info ./server/target/release/simples --sedes ./config/americas.toml --secret ./config/americas_secret.toml -v ./www/:  -v ./www-transparent/:  ${BOCA_URL}
