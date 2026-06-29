//! UbiHome Builder engine.
//!
//! Shared core used by both the `ubihome-builder` CLI and the
//! `ubihome-builder-server` web dashboard. It is fully decoupled from the
//! UbiHome repository it may ship next to: it clones UbiHome on demand and
//! builds any tagged version in an isolated git worktree. It knows how to:
//!
//! - manage a UbiHome clone and list its versions ([`git`]),
//! - detect which platform components a `config.yml` references ([`platforms`]),
//! - figure out which compile targets are feasible on this host ([`targets`]),
//! - compile a slim UbiHome binary with only those components ([`compile`]),
//! - validate a config using the real UbiHome validator ([`validate`]),
//! - persist multiple configs and a build history ([`store`]).
//!
//! It has no web dependencies, so the CLI stays lean.

pub mod compile;
pub mod git;
pub mod platforms;
pub mod store;
pub mod targets;
pub mod validate;

/// Errors surfaced by the engine.
#[derive(Debug, thiserror::Error)]
pub enum BuilderError {
    /// Problem with the UbiHome source / git / Cargo.toml.
    #[error("source error: {0}")]
    Source(String),
    /// Problem with the user's config (missing/unknown components, bad name).
    #[error("config error: {0}")]
    Config(String),
    /// The compile step failed.
    #[error("build error: {0}")]
    Build(String),
    /// Validation could not be run.
    #[error("validate error: {0}")]
    Validate(String),
}

pub use compile::{build, BuildArtifact, BuildOptions};
pub use git::{default_cache_root, Repo, DEFAULT_REPO_URL};
pub use platforms::{available_components, detect_platforms};
pub use store::{BuildRecord, ConfigInfo, Store};
pub use targets::{feasible_targets, host_triple, Target};
pub use validate::{validate as validate_config, ValidationResult};
