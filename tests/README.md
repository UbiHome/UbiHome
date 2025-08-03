# E2E Testing

## Installation

```
pipx install poetry
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