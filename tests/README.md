# E2E Testing

## Installation

1. Install [`uv`](https://docs.astral.sh/uv/getting-started/installation/)

```bash
cd tests
uv sync
```

## Just run them

```bash
make prepare-test-linux
make test
```
## Development

```bash
make prepare-test-linux
cd tests
eval $(poetry env activate)
pytest
```

## If something is not working

```bash
# Linux
pkill -8  ubihome
# Check that no processes are running
ps aux | grep ubihome
```