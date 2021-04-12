.SILENT:
.PHONY:

ENCRYPT_TEST_ENV:
	gpg -a -r 0x0BD10E4E6E578FB6 -o .env.asc -e .env

DECRYPT_TEST_ENV:
	rm -rf .env
	gpg -a -r 0x0BD10E4E6E578FB6 -o .env -d .env.asc

TEST:
	cargo test -- --nocapture