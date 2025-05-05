.PHONY: prepare-test-linux test

prepare-test-linux:
	cargo build --release
	cp target/release/ubihome ./tests/ubihome

prepare-test-linux-fast:
	cargo build
	cp target/debug/ubihome ./tests/ubihome

test:
	cd tests && poetry run pytest -vvv
