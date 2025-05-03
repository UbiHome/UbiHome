.PHONY: prepare-test-linux test

prepare-test-linux:
	cargo build --release
	cp target/release/ubihome ./tests/ubihome

test:
	cd tests && poetry run pytest -vvv
