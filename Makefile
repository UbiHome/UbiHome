.PHONY: prepare-test-linux test prepare-test-linux-fast

prepare-test-linux:
	cargo build --release
	cp target/release/ubihome ./tests/ubihome

prepare-test-linux-fast:
	cargo build
	cp target/debug/ubihome ./tests/ubihome

test: prepare-test-linux-fast
	cd tests && poetry run pytest -vvv
