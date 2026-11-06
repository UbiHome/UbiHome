# Home Assistant Playwright E2E tests

These tests launch a real Home Assistant Docker container, connect it to a real `ubihome` process through the native API (ESPHome integration), and verify UI behavior.

## Run

```bash
cd tests
uv run --locked playwright install chromium
uv run --locked pytest -vvv -m e2e e2e/home_assistant_native_api_e2e_test.py

# Run in headed mode to watch the browser actions
uv run --locked pytest -vvv -m e2e --headed e2e/home_assistant_native_api_e2e_test.py

# Save Playwright traces only for failing tests
uv run --locked pytest -vvv -m e2e --trace-on-failure e2e/home_assistant_native_api_e2e_test.py
```

## Manual testing

```
docker run --rm --name home-assistant -p 8123:8123 -v /home/codespace/.homeassistant:/config homeassistant/home-assistant:2026.4.3
```


## Notes

- No components are mocked.
- Failed test traces are written to `tests/test-results/` when `--trace-on-failure` is enabled.
