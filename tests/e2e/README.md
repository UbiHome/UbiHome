# Home Assistant Playwright E2E tests

These tests launch a real Home Assistant Docker container, connect it to a real `ubihome` process through the native API (ESPHome integration), and verify UI behavior.

## Run

```bash
cd tests
uv run --locked playwright install chromium
uv run --locked pytest -vvv -m e2e e2e/home_assistant_native_api_e2e_test.py
```

## Notes

- No components are mocked.
- The suite is skipped by default unless `RUN_PLAYWRIGHT_E2E=1` is set.
