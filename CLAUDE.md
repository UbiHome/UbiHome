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
- Documentation site: `documentation/` (Astro Starlight, built/checked with Dagger).

## Key Entry Points

- CLI entrypoint: `src/main.rs`
  - Commands: `run`, `validate`, `install`, `update`, `uninstall`.
  - Default config lookup: `config.yml` then `config.yaml`.
- Runtime orchestration: `src/commands/run.rs`
  - Reads config file content.
  - Parses + validates the whole file once into the generated `RootConfig` (`src/components.rs`).
  - Builds the configured platform modules from it and runs them on tokio tasks.
- Whole-file validation (platform availability, duplicate ids, action/lambda references): `src/config.rs`.

## Dynamic Platform Wiring

- `build.rs` discovers dependencies prefixed with `ubihome-` (except `ubihome-core`) from root `Cargo.toml`.
- It generates `components.rs` into `OUT_DIR`; root `src/components.rs` includes it.
- `generate_component_methods!` in `src/components.rs` turns that list into a single `RootConfig`: one
  optional section per platform, and per entity kind an enum dispatched on `platform:`
  (`ubihome_core::platform::visit_platform_map`, which streams instead of buffering to keep error locations).
- Do not edit generated `OUT_DIR/components.rs` manually.
- If adding/removing platform crates, update root `Cargo.toml` dependency/workspace entries and rebuild.

## Component Pattern (Important)

Most platform crates follow this model:

1. Define platform-specific config structs (`serde::Deserialize` + `garde::Validate`).
2. Use `ubihome_core::config_template!` to declare the config types and generate `CoreConfig`
   (implements `ubihome_core::platform::PlatformConfigTypes`).
3. Implement `ubihome_core::platform::ConfigurableModule` for `UbiHomePlatform`:
   `new(config: CoreConfig)` receives the already validated config (crates never parse YAML).
   In tests use `UbiHomePlatform::from_yaml(...)`.
4. Implement `ubihome_core::Module` with:
   - `components()` for discovered entities.
   - `run(sender, receiver)` for async behavior.

Shared primitives/macro locations:

- `components/core/src/lib.rs` (Module trait, message enums, `config_template!`).
- `components/core/src/platform.rs` (single-pass config traits, `platform:` dispatch).
- `components/core/src/validation.rs` (`validate_config`, standalone `parse_platform_config`).

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
- The file is parsed exactly once: `ubihome_core::validation::validate_config::<RootConfig>(...)` in `src/commands/run.rs`.
- Cross-entity checks live in the hand-written `Validate` for `RootConfig` (`src/config.rs`) and report errors via
  garde paths, so they get source locations like every other field error.

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

## Documentation Guidelines

Deduced from review feedback; apply to docs changes.

- Full, end-to-end configuration examples belong in `documentation/src/content/docs/examples/<name>/index.md` (one dir + `index.md`), not inline in platform/component pages.
- Platform/component pages should reference shared concept pages (e.g. Triggers and Actions, Binary Sensor entity) instead of duplicating their tables, option lists, or explanations.
- Cross-references are surfaced via the hidden backlinks `<div>` at the bottom of a page — add relevant pages there (including shared concept pages), and don't also add inline "See the X example" prose.
- Doc pages use frontmatter `title:` (plus `tags:` where relevant); do not add a top-level `# H1`. Section headings start at `##`, subsections at `###`, and nest deeper (`####`) for details only relevant under a parent section.
- Be concise: cut prose that merely restates what a linked page already covers or what is shown just above/below it.
- Only document platform-specific options on the platform page; link the entity/concept page for everything shared.
- Fold short explanatory notes into the example itself (e.g. as YAML comments) rather than leaving them as separate standalone sentences.
- When trimming prose, keep the useful links (e.g. to the Filters page) rather than dropping them with the sentence.

## Pull Request Guidelines

PRs are squash-merged: the PR title becomes the commit subject and the PR description becomes the commit body, which ends up in the release notes.

- Title: a Conventional Commit (`feat(sendspin): …`, `fix(api): …`); it is the changelog entry.
- Description: written for users reading the release notes. Say what changed and what it means for them: new options (with a short YAML example and the default), behaviour changes, docs added. No testing logs, investigation notes or follow-up ideas.
- Put everything else in the first comment directly below the description: how it was tested (commands, results, devices), background and root cause, design decisions, and follow-ups or known issues outside the PR.
- When the scope changes (e.g. an option is renamed), update the description to match the final state instead of appending to it.

## Useful Paths

- Root binary: `src/main.rs`
- Runtime: `src/commands/run.rs`
- Base config: `src/config.rs`
- Dynamic platform include: `src/components.rs`
- Build codegen: `build.rs`
- Core trait/macro: `components/core/src/lib.rs`
- Single-pass config support: `components/core/src/platform.rs`
- Test docs: `tests/README.md`
- Docs build docs: `documentation/README.md`

## Way of work

Try to use subagents with less expensive models (sonnet / haiku) where it makes sense.
