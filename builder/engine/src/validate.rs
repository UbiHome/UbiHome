//! Config validation. Reuses UbiHome's real validator by building (once per
//! version) a full `ubihome` binary from the requested tag and running
//! `ubihome -c <config> validate`. The result is the identical `serde_saphyr` +
//! `garde` reporting the device sees. The validator binary is cached per
//! version, so only the first validation of a given version compiles.

use std::path::PathBuf;
use std::process::Stdio;

use crate::git::Repo;
use crate::BuilderError;

/// Outcome of validating a config.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationResult {
    pub ok: bool,
    /// Combined stdout+stderr from `ubihome validate` (the diagnostic report).
    pub output: String,
    /// Resolved version the config was validated against.
    pub version: String,
}

/// Ensure a full `ubihome` validator binary exists for `reference`, building it
/// if needed. Returns the cached binary path.
async fn ensure_validator(repo: &Repo, reference: &str) -> Result<PathBuf, BuilderError> {
    let bin = repo.validator_bin(reference);
    if bin.is_file() {
        return Ok(bin);
    }
    let label = format!("{reference}-validate");
    let workdir = repo.worktree(&label, reference)?;

    // Full build (no Cargo.toml trim) → a validator that knows every platform.
    let exe = if cfg!(windows) {
        "ubihome.exe"
    } else {
        "ubihome"
    };
    let output = tokio::process::Command::new("cargo")
        .current_dir(&workdir)
        .env("CARGO_HOME", repo.cargo_home())
        .env("CARGO_TARGET_DIR", repo.target_dir(&label))
        .args(["build", "--release", "--bin", "ubihome"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| BuilderError::Validate(format!("failed to run cargo: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let lines: Vec<&str> = stderr.lines().collect();
        let tail = lines[lines.len().saturating_sub(20)..].join("\n");
        return Err(BuilderError::Validate(format!(
            "failed to build validator for {reference}:\n{tail}"
        )));
    }

    let built = repo.target_dir(&label).join("release").join(exe);
    std::fs::create_dir_all(bin.parent().unwrap())
        .map_err(|e| BuilderError::Validate(format!("cannot create bin dir: {e}")))?;
    std::fs::copy(&built, &bin)
        .map_err(|e| BuilderError::Validate(format!("cannot cache validator: {e}")))?;
    Ok(bin)
}

/// Validate a config against a specific version (`None` = latest stable). The
/// config is written to a temp file and checked with the cached validator.
pub async fn validate(
    repo: &Repo,
    reference: Option<&str>,
    config: &str,
) -> Result<ValidationResult, BuilderError> {
    let resolved = repo.resolve(reference)?;
    let bin = ensure_validator(repo, &resolved).await?;

    let mut tmp: PathBuf = std::env::temp_dir();
    tmp.push(format!(
        "ubihome-builder-validate-{}-{}.yml",
        std::process::id(),
        config.len()
    ));
    tokio::fs::write(&tmp, config)
        .await
        .map_err(|e| BuilderError::Validate(format!("cannot write temp config: {e}")))?;

    // Note: `-c/--configuration` is a global flag and must precede `validate`.
    let result = tokio::process::Command::new(&bin)
        .arg("-c")
        .arg(&tmp)
        .arg("validate")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    let _ = tokio::fs::remove_file(&tmp).await;

    let output =
        result.map_err(|e| BuilderError::Validate(format!("could not run validator: {e}")))?;
    let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&output.stderr));

    Ok(ValidationResult {
        ok: output.status.success(),
        output: combined.trim().to_string(),
        version: resolved,
    })
}

/// Path helper exposed for callers that want to check cache presence.
pub fn validator_path(repo: &Repo, reference: &str) -> PathBuf {
    repo.validator_bin(reference)
}
