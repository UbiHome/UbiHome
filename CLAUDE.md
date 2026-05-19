# CLAUDE.md

## Purpose
This file is a working guide for coding agents in the UbiHome monorepo.
Prioritize safe, minimal changes, and validate behavior with targeted tests.

## Repository At A Glance
- Language/runtime: Rust workspace (main product) plus Python test harness.
- Main binary crate: `ubihome` at repository root.
- Workspace crates: `components/*` (platform modules).
- Core shared crate: `components/core` (traits, config templates, mapping macros, common types).
- Python tests: `tests/` (pytest + uv, includes API, MQTT, e2e and platform tests).
- Documentation site: `documentation/` (MkDocs Material, built/checked with Dagger).

## Key Entry Points
- CLI entrypoint: `src/main.rs`
  - Commands: `run`, `validate`, `install`, `update`, `uninstall`.
  - Default config lookup: `config.yml` then `config.yaml`.
- Runtime orchestration: `src/commands/run.rs`
  - Reads config file content.
  - Extracts configured top-level platforms.
  - Validates with `serde_saphyr` + `garde` using `BaseConfig` context.
  - Dynamically loads platform modules and runs them on tokio tasks.
- Base config + platform discovery: `src/config.rs`.

## Dynamic Platform Wiring
- `build.rs` discovers dependencies prefixed with `ubihome-` (except `ubihome-core`) from root `Cargo.toml`.
- It generates `components.rs` into `OUT_DIR`; root `src/components.rs` includes it.
- Do not edit generated `OUT_DIR/components.rs` manually.
- If adding/removing platform crates, update root `Cargo.toml` dependency/workspace entries and rebuild.

## Component Pattern (Important)
Most platform crates follow this model:
1. Define platform-specific config structs (`serde::Deserialize` + `garde::Validate`).
2. Use `ubihome_core::config_template!` to produce a combined `CoreConfig`.
3. Implement `ubihome_core::Module` for `UbiHomePlatform` with:
   - `new(config_string)` for parsing/validation.
   - `components()` for discovered entities.
   - `run(sender, receiver)` for async behavior.

Shared primitives/macro locations:
- `components/core/src/lib.rs` (Module trait, message enums, `config_template!`).
- `components/core/src/mapper.rs` (`template_mapper_new!` for mapping lists into keyed maps).

## Build, Lint, Test Commands
Use the smallest command set needed for your change.

### Rust
- Build debug: `cargo build`
- Build release: `cargo build --release`
- Format check: `cargo fmt --all -- --check`
- Lint (deny warnings): `cargo clippy --all-targets --all-features -- -D warnings`

### Python tests
From `tests/`:
- Setup env: `uv sync`
- Run all tests (after binary copied): `uv run pytest -vvv`
- Run targeted test: `uv run pytest basic/config_test.py::test_validate_config -xvv`

From repo root helper targets:
- `make prepare-test-linux-fast` (copies debug binary to `tests/ubihome`)
- `make test-fast`
- `make prepare-test-linux` (copies release binary)
- `make test`

### Documentation
From repo root:
- Preview: `dagger call docs-preview up`
- Strict checks: `dagger check`
- Build static docs: `dagger call docs-build-dir export --path ./documentation/site`

## Configuration + Validation Notes
- The project uses `serde-saphyr` for YAML parsing and rich validation/reporting.
- Runtime validation path is in `src/commands/run.rs` using:
  - `serde_saphyr::from_str_with_options_context_valid::<BaseConfig>(...)`
  - `serde_saphyr::miette::to_miette_report(...)`
- Top-level platform extraction currently relies on line-based parsing in `get_platforms_from_config`; be careful when changing YAML shape assumptions.

## Change Strategy For Agents
- Keep changes scoped and avoid broad refactors unless explicitly requested.
- Prefer modifying one platform/module at a time and validating with targeted tests.
- If touching config parsing/validation, always run at least one config-focused pytest and one Rust check.
- Do not revert unrelated local changes in dirty worktrees.

## Quick Start Checklist For A New Task
1. Read the relevant crate and root orchestration files.
2. Identify whether the change is in:
   - core/shared behavior,
   - one platform crate,
   - CLI/runtime wiring,
   - tests/docs.
3. Make minimal edits.
4. Run focused validation:
   - Rust: `cargo check` or `cargo clippy` as appropriate.
   - Python: targeted `uv run pytest ...` when behavior is covered there.
5. Summarize risks and what was/was not tested.

## Useful Paths
- Root binary: `src/main.rs`
- Runtime: `src/commands/run.rs`
- Base config: `src/config.rs`
- Dynamic platform include: `src/components.rs`
- Build codegen: `build.rs`
- Core trait/macro: `components/core/src/lib.rs`
- Mapper macro: `components/core/src/mapper.rs`
- Test docs: `tests/README.md`
- Docs build docs: `documentation/README.md`
