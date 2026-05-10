# [Documentation](https://ubihome.github.io/) Repository for UbiHome

Use Dagger for docs preview, checks, and builds.

Install Dagger:

```bash
curl -fsSL https://dl.dagger.io/dagger/install.sh | BIN_DIR="$HOME/.local/bin" sh
```

Run a local preview:

```bash
dagger -c "docs-preview | up"
```

Run strict checks:

```bash
dagger check
```

Build static docs output:

```bash
dagger call docs-build-dir export --path ./documentation/site
```

Visit [http://localhost:8000](http://localhost:8000) to see the documentation preview.

## Links

- [Icons and Emojis](https://squidfunk.github.io/mkdocs-material/reference/icons-emojis/)