# UbiHome Builder

An [esphome-builder](https://github.com/esphome/esphome)-style tool for UbiHome.
Give it a `config.yml` and it compiles a **slim UbiHome binary that contains only
the platform components your config actually uses** — smaller binary, smaller
attack surface.

It is **fully decoupled** from the UbiHome repository: it **clones UbiHome on
demand** and builds **any tagged version** in an isolated `git worktree`. Nothing
in your checkout is touched. By default it builds the **latest stable tag**
(pre-releases like `-next` are ignored); you can pick any tag/branch/commit.

Two forms share one engine:

- **`ubihome-builder`** — a lean CLI (no web dependencies). Run it natively to
  build a binary for your own OS.
- **`ubihome-builder-server`** — a web dashboard (the Docker deployment) to
  manage multiple configs, validate them, pick a version, build with live
  streaming logs, and keep a build history.

## How it works

UbiHome's `build.rs` derives its component registry purely from the `ubihome-*`
dependencies in the root `Cargo.toml`. For each build the engine:

1. clones UbiHome (cached) and resolves the requested version (default: latest
   stable tag);
2. materializes that version as an isolated `git worktree`;
3. rewrites *that throwaway copy's* `Cargo.toml` to keep `ubihome-core` + only the
   detected components;
4. runs `cargo build --release` and copies out the binary.

Components are detected exactly like the runtime does (every top-level YAML key
that is not a base field such as `ubihome:`, `logger:`, `sensor:`, …). No UbiHome
source changes are required, and no source is baked into the image.

## Quick start — dashboard (Docker)

```bash
cd builder
docker compose up --build
# open http://localhost:8080
```

- The image contains only the builder + the Rust toolchain. On first use it
  **clones UbiHome and compiles** (cached in the `ubihome-builder-cache` volume);
  later builds are fast.
- Configs, built binaries and history persist under `builder/data/`.
- Point at a different repo/fork with `BUILDER_REPO_URL`.

## Quick start — CLI (native)

The CLI builds for **your host OS** using your local Rust toolchain — this is how
a macOS or Windows user produces a native binary (a Linux Docker container can
only emit Linux/ARM binaries).

```bash
cd builder
cargo build --release -p ubihome-builder
B=./target/release/ubihome-builder
$B detect   -c ../config.yml          # show components (local, no clone)
$B targets                            # buildable targets on this host
$B versions                           # buildable UbiHome versions (stable tags)
$B validate -c ../config.yml          # validate against latest stable
$B build    -c ../config.yml -o ./output            # build latest stable
$B build    -c ../config.yml -r v0.14.0 -o ./output # build a specific version
```

Global options: `--repo-url <url|path>` (env `BUILDER_REPO_URL`, default the
official repo) and `--work <dir>` (env `BUILDER_WORK`, default
`$HOME/.cache/ubihome-builder`) for the clone/worktree/cargo cache.

## Versions

`versions` lists the stable tags (newest first). Builds default to the latest
stable tag; override per build with `-r/--ref <tag|branch|sha>` (CLI) or the
version dropdown (dashboard). The chosen version is recorded in the artifact name
(`ubihome-<version>-<os>-<arch>`) and in build history.

## Build outputs & history

Artifacts are named `ubihome-<config>-<version>-<os>-<arch>` so different configs
never collide on filename.

- **Dashboard:** every build is a separate history entry with its own
  `data/output/<build-id>/` directory. Editing a config and rebuilding creates a
  *new* entry — previous builds are preserved and stay downloadable from History.
- **CLI:** writes the artifact straight into `-o` with no history, so rebuilding
  the same config+version+target overwrites that file (normal for a CLI). Use a
  different `-o`, or the dashboard, if you want to keep older builds.

## Validation

Validation runs UbiHome's real validator (`serde_saphyr` + `garde`), identical to
the device. The engine builds a full `ubihome` for the selected version once and
caches it, so only the **first** validation of a given version compiles.

## Targets & cross-compilation

`targets` lists what is feasible on the current host: always the **host triple**,
and on Linux the **ARM musl** (Raspberry Pi) targets when reachable — either an
installed `rustup` target or via [`cross`](https://github.com/cross-rs/cross)
(which uses the repo's `Cross.toml`). macOS/Windows binaries must be built
natively with the CLI on that OS.

## Configuration (env)

| Var | Default | Meaning |
|---|---|---|
| `BUILDER_REPO_URL` | official repo | UbiHome git repo (URL or local path) to build from |
| `BUILDER_WORK` | `./cache` (server) / `$HOME/.cache/ubihome-builder` (CLI) | clone + worktrees + cargo cache |
| `BUILDER_DATA` | `./data` | configs, outputs, logs, history (server) |
| `BUILDER_BIND` | `0.0.0.0:8080` | server bind address |

## Layout

```
builder/
  engine/    shared core (git, detect, trim Cargo.toml, compile, validate, store) — no web deps
  cli/       ubihome-builder         (engine + clap only)
  server/    ubihome-builder-server  (engine + axum + embedded Angular SPA)
  frontend/  Angular dashboard
  Dockerfile, docker-compose.yml
```

## REST API (served under `/api`)

| Method | Path | Purpose |
|---|---|---|
| GET | `/api/targets` | feasible compile targets |
| GET | `/api/versions` | buildable versions (stable tags) |
| GET/POST | `/api/configs` | list / create configs |
| GET/PUT/DELETE | `/api/configs/:name` | read / save / delete a config |
| POST | `/api/configs/:name/validate?ref=` | validate against a version |
| POST | `/api/configs/:name/duplicate` · `/rename` | manage configs |
| POST | `/api/configs/:name/build` | start a build (`{target?, ref?}`) → `{build_id}` |
| GET | `/api/builds` · `/api/builds/:id` | build history |
| WS | `/api/builds/:id/logs` | live build log stream |
| GET | `/api/builds/:id/log` · `/api/builds/:id/artifact` | log / download binary |

## Development

```bash
# backend (terminal 1) — uses a local clone of any UbiHome repo
cargo run -p ubihome-builder-server -- \
  --bind 127.0.0.1:8080 --data ./data --work ./cache \
  --repo-url https://github.com/UbiHome/UbiHome.git
# frontend with live reload + API proxy (terminal 2)
cd frontend && npm install && npm start   # http://localhost:4200
```

Tests: `cargo test` (in `builder/`) — includes the component-detection rule,
Cargo.toml trimming, stable-version parsing, and a drift guard that parses a real
UbiHome `Cargo.toml`.
