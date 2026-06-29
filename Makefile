.PHONY: prepare-test-linux test prepare-test-linux-fast lint \
	builder-image builder-serve builder-test builder-build

# --- UbiHome Builder (esphome-builder style) ---------------------------------
# Build the dashboard Docker image (context is builder/, fully decoupled from the repo).
builder-image:
	docker build -f builder/Dockerfile -t ubihome-builder builder

# Run the dashboard at http://localhost:8080 (builds the image if needed).
builder-serve:
	cd builder && docker compose up --build

# Run the builder Rust tests (engine unit tests + drift guard).
builder-test:
	cd builder && cargo test

# Build a slim binary from ./config.yml using the native CLI.
builder-build:
	cd builder && cargo run --release -p ubihome-builder -- build -c ../config.yml -o ./output

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

lint:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings