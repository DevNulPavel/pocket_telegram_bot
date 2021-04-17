.SILENT:
.PHONY:

ENCRYPT_TEST_ENV:
	gpg -a -r 0x0BD10E4E6E578FB6 -o env_encrypted/local.env.asc -e env/local.env
	gpg -a -r 0x0BD10E4E6E578FB6 -o env_encrypted/docker.env.asc -e env/docker.env

DECRYPT_TEST_ENV:
	rm -rf .env
	gpg -a -r 0x0BD10E4E6E578FB6 -o env/local.env -d env_encrypted/env.asc
	gpg -a -r 0x0BD10E4E6E578FB6 -o env/docker.env -d env_encrypted/docker.env.asc

TEST:
	cargo test -- --nocapture

DOCKER_BUILD_AND_PUSH:
	# docker buildx create --use
	# docker buildx build --platform linux/amd64,linux/arm64 -t devnul/pocket_telegram_bot .
	# docker buildx rm
	docker build -t devnul/pocket_telegram_bot .
	docker push devnul/pocket_telegram_bot

DOCKER_RUST_BASH:
	docker run -it rust:1.51.0-alpine sh

DOCKER_RUN:
	docker-compose up