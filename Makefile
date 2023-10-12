all:
	@echo build-server
	@echo build-client
	@echo build-client-ccl

build-server:
	docker build \
		-t wuerges/animeitor server
	docker tag wuerges/animeitor wuerges/animeitor:0.7.0
	docker tag wuerges/animeitor wuerges/animeitor:latest
	docker push wuerges/animeitor:0.7.0
	docker push wuerges/animeitor:latest
	
build-client:
	docker build -f client.Dockerfile \
		--build-arg "URL_PREFIX=http://animeitor.naquadah.com.br:8000" \
		-t wuerges/animeitor-client .
	docker tag wuerges/animeitor-client wuerges/animeitor-client:0.7.0
	docker tag wuerges/animeitor-client wuerges/animeitor-client:latest
	docker push wuerges/animeitor-client:0.7.0
	docker push wuerges/animeitor-client:latest

build-client-ccl:
	docker build -f client.Dockerfile \
		--build-arg "URL_PREFIX=http://animeitor.naquadah.com.br:8001" \
		-t wuerges/animeitor-client-ccl .
	docker tag wuerges/animeitor-client wuerges/animeitor-client:0.7.0
	docker tag wuerges/animeitor-client wuerges/animeitor-client:latest
	docker push wuerges/animeitor-client:0.7.0
	docker push wuerges/animeitor-client:latest
