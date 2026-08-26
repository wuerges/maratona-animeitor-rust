.PHONY: build-client upload-client set-endpoint-as-www-bucket upload-dummy-photo upload-dummy-sound


all: build-client upload-client
	@echo done

build-client:
	( cd ../client-v2 && \
		trunk build --release -d bucket )
	cp serve-as-bucket/config.json ../client-v2/bucket/config.json

upload-client:
	s3cmd put --recursive ../client-v2/bucket s3://static.animeitor/
	s3cmd put -m text/css ../client-v2/bucket/*.css s3://static.animeitor/
	s3cmd setacl --acl-public --recursive s3://static.animeitor/

upload-dummy-photo:
	s3cmd put ../server/photos/fake.webp s3://static.animeitor/photos/fake.webp
	s3cmd setacl --acl-public s3://static.animeitor/photos/fake.webp

upload-dummy-sound:
	s3cmd put ../server/sounds/applause.mp3 s3://static.animeitor/sounds/applause.mp3
	s3cmd setacl --acl-public s3://static.animeitor/sounds/applause.mp3

set-endpoint-as-www-bucket:
	AWS_PROFILE=maratona aws s3api put-bucket-website \
		--bucket static.animeitor \
		--endpoint=https://nyc3.digitaloceanspaces.com \
		--website-configuration file://website.json
