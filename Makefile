.PHONY: prepare-test-linux test prepare-test-linux-fast

prepare-test-linux:
	cargo build --release
	cp target/release/ubihome ./tests/ubihome

prepare-test-linux-fast:
	cargo build
	cp target/debug/ubihome ./tests/ubihome

test-fast: prepare-test-linux-fast
	cd tests && uv run pytest -vvv

test: prepare-test-linux
	cd tests && uv run pytest -vvv
