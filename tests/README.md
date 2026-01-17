# E2E Testing

## Installation

1. Install [`uv`](https://docs.astral.sh/uv/getting-started/installation/)

```bash
cd tests
uv sync
```

## Linux

## Just run them

```bash
make prepare-test-linux
make test
cd tests
uv run pytest
```

## If something is not working

```bash
# Linux
pkill -8  ubihome
# Check that no processes are running
ps aux | grep ubihome
```

## Windows

```powershell
cd tests
cp ..\target\debug\ubihome.exe .\ubihome.exe
uv run pytest
```
