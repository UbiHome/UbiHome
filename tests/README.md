# E2E Testing

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
