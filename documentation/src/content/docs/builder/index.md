---
title: 'Builder'
description: Build a slim UbiHome binary containing only the components your config uses, from any tagged version.
---

The **UbiHome Builder** is an [esphome-builder](https://github.com/esphome/esphome)-style
tool. You give it a `config.yml` and it compiles a UbiHome binary that contains
**only the platform components your configuration actually uses** — a smaller
binary with a smaller attack surface.

It is **fully decoupled** from the UbiHome source: it **clones UbiHome on demand**
and builds **any tagged version** in an isolated `git worktree`. By default it
builds the **latest stable tag** (pre-releases such as `-next` are ignored); you
can select any tag, branch or commit.

It works because UbiHome's component registry is generated from the `ubihome-*`
dependencies in `Cargo.toml`. The builder detects the components referenced by
your config, keeps only those (plus the core) in a throwaway worktree, and
compiles. No source changes needed; nothing in your checkout is touched.
Components are detected exactly like the runtime does: every top-level key that is
not a base field (`ubihome:`, `logger:`, `sensor:`, `button:`, …).

It ships in two forms that share one engine:

- **`ubihome-builder`** — a lean CLI (no web dependencies) for building on your
  own machine.
- **`ubihome-builder-server`** — a web dashboard to manage multiple configs,
  validate them, pick a version, build with live streaming logs, and keep a
  build history.

## Dashboard (Docker)

```bash
cd builder
docker compose up --build
# open http://localhost:8080
```

The image contains only the builder + the Rust toolchain. On first use it clones
UbiHome and compiles (cached in a Docker volume); later builds are fast. Configs,
binaries and history persist under `builder/data/`. Point at a fork with
`BUILDER_REPO_URL`.

## CLI (native)

The CLI builds for **your host OS** using your local Rust toolchain. This is how
macOS and Windows users get a native binary — a Linux Docker container can only
emit Linux/ARM binaries.

```bash
cd builder
cargo build --release -p ubihome-builder
B=./target/release/ubihome-builder
$B detect   -c ../config.yml                         # show components
$B versions                                          # buildable versions (stable tags)
$B targets                                           # buildable targets
$B validate -c ../config.yml                         # validate against latest stable
$B build    -c ../config.yml -o ./output             # build latest stable
$B build    -c ../config.yml -r v0.14.0 -o ./output  # build a specific version
```

Global options: `--repo-url <url|path>` (env `BUILDER_REPO_URL`) and `--work <dir>`
(env `BUILDER_WORK`) control which repo is built and where the clone/cache live.

## Versions & targets

Builds default to the latest stable tag; override with `-r/--ref` (CLI) or the
version dropdown (dashboard). The version is recorded in the artifact name
(`ubihome-<version>-<os>-<arch>`) and in build history.

`targets` offers the host target and, on Linux, the ARM (Raspberry Pi) musl
targets when reachable via `rustup` or [`cross`](https://github.com/cross-rs/cross).
macOS/Windows binaries must be built natively with the CLI on that OS.

See `builder/README.md` for the full REST API and development workflow.
