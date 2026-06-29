//! The core build step: turn a `config.yml` into a slim UbiHome binary that
//! contains only the platform components the config uses, built from a chosen
//! tagged version of the UbiHome repository.
//!
//! Mechanism (no UbiHome source changes required): UbiHome's `build.rs` derives
//! its component registry purely from the `ubihome-*` entries in the root
//! `Cargo.toml`. We materialize the requested version as an isolated git
//! worktree (see [`crate::git`]), rewrite *that throwaway copy's* `Cargo.toml`
//! to keep `ubihome-core` plus only the selected components, and compile it.
//! Nothing shared is mutated.

use std::path::PathBuf;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;

use crate::git::Repo;
use crate::platforms;
use crate::targets;
use crate::BuilderError;

/// Builds share the clone's worktrees and cargo cache, so serialize them.
static BUILD_LOCK: Mutex<()> = Mutex::const_new(());

/// Options for a single build.
#[derive(Debug, Clone)]
pub struct BuildOptions {
    /// The managed UbiHome clone (url + cache root).
    pub repo: Repo,
    /// Version/ref to build (tag, branch, or commit). `None` = latest stable tag.
    pub reference: Option<String>,
    /// The user's config.yml contents (used only to detect components).
    pub config: String,
    /// Optional name (e.g. the config file/node name) used to make the output
    /// filename distinct per config. `None` omits it.
    pub name: Option<String>,
    /// Where to place the finished binary. Callers that build many configs/
    /// versions should make this unique per build (e.g. a per-build subdir) so
    /// artifacts never overwrite each other.
    pub output_dir: PathBuf,
    /// Target triple to build for. `None` = native host build.
    pub target: Option<String>,
    /// Use the `cross` tool instead of `cargo` (for ARM musl targets).
    pub use_cross: bool,
}

/// Result of a successful build.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BuildArtifact {
    /// Final path of the copied binary.
    pub path: PathBuf,
    /// Size in bytes.
    pub size: u64,
    /// Components compiled into the binary (sorted).
    pub components: Vec<String>,
    /// Resolved version/ref that was built.
    pub version: String,
    /// Target triple, or "host".
    pub target: String,
    /// Artifact suffix, e.g. `macos-aarch64`.
    pub artifact: String,
}

/// Turn an optional config name into a filename-safe slug (drops a trailing
/// `.yml`/`.yaml`, replaces unsafe chars). Returns None if empty.
fn name_part(name: Option<&str>) -> Option<String> {
    let raw = name?.trim();
    let raw = raw
        .strip_suffix(".yml")
        .or_else(|| raw.strip_suffix(".yaml"))
        .unwrap_or(raw);
    let slug: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if slug.is_empty() {
        None
    } else {
        Some(slug)
    }
}

/// Generate the trimmed Cargo.toml content: keep every non-`ubihome-*`
/// dependency plus `ubihome-core`, and keep `ubihome-<c>` only for `c` in
/// `selected`. Everything else (workspace, profiles, etc.) is left untouched.
pub fn trim_cargo_toml(original: &str, selected: &[String]) -> Result<String, BuilderError> {
    let mut doc: toml_edit::DocumentMut = original
        .parse()
        .map_err(|e| BuilderError::Source(format!("invalid Cargo.toml: {e}")))?;
    let deps = doc
        .get_mut("dependencies")
        .and_then(|d| d.as_table_mut())
        .ok_or_else(|| BuilderError::Source("Cargo.toml has no [dependencies]".into()))?;

    let keep: std::collections::HashSet<String> =
        selected.iter().map(|c| format!("ubihome-{c}")).collect();

    let remove: Vec<String> = deps
        .iter()
        .map(|(k, _)| k.to_string())
        .filter(|k| k.starts_with("ubihome-") && k != "ubihome-core" && !keep.contains(k))
        .collect();

    for key in remove {
        deps.remove(&key);
    }
    Ok(doc.to_string())
}

/// Build the `cargo`/`cross` command for compiling the `ubihome` binary in
/// `workdir`, with caches pointed at the repo's shared/per-version dirs.
fn cargo_build_command(
    repo: &Repo,
    workdir: &std::path::Path,
    target_label: &str,
    target: &Option<String>,
    use_cross: bool,
) -> tokio::process::Command {
    let program = if use_cross { "cross" } else { "cargo" };
    let mut cmd = tokio::process::Command::new(program);
    cmd.current_dir(workdir)
        .env("CARGO_HOME", repo.cargo_home())
        .env("CARGO_TARGET_DIR", repo.target_dir(target_label))
        .arg("build")
        .arg("--release")
        .arg("--bin")
        .arg("ubihome");
    if let Some(triple) = target {
        cmd.arg("--target").arg(triple);
    }
    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    cmd
}

/// Stream a child process's stdout+stderr to `log` and wait for it to exit.
async fn run_streaming(
    mut child: tokio::process::Child,
    log: &UnboundedSender<String>,
) -> Result<std::process::ExitStatus, BuilderError> {
    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");

    let log_out = log.clone();
    let out_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = log_out.send(line);
        }
    });
    let log_err = log.clone();
    let err_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = log_err.send(line);
        }
    });

    let status = child
        .wait()
        .await
        .map_err(|e| BuilderError::Build(format!("build process error: {e}")))?;
    let _ = out_task.await;
    let _ = err_task.await;
    Ok(status)
}

/// Path to the `ubihome` binary cargo produced for the given target.
fn built_binary(repo: &Repo, target_label: &str, target: &Option<String>) -> (PathBuf, bool) {
    let root = repo.target_dir(target_label);
    let is_windows = target
        .as_deref()
        .map(|t| t.contains("windows"))
        .unwrap_or(cfg!(windows));
    let bin = if is_windows { "ubihome.exe" } else { "ubihome" };
    let path = match target {
        Some(t) => root.join(t).join("release").join(bin),
        None => root.join("release").join(bin),
    };
    (path, is_windows)
}

/// Run a build end to end. Log lines (clone/checkout progress + interleaved
/// cargo output) are sent to `log`. Serializes against other builds.
pub async fn build(
    opts: BuildOptions,
    log: UnboundedSender<String>,
) -> Result<BuildArtifact, BuilderError> {
    // Detect the component set up front (cheap; needs no source).
    let selected = platforms::detect_platforms(&opts.config);
    if selected.is_empty() {
        return Err(BuilderError::Config(
            "no platform components found in config (nothing to build)".into(),
        ));
    }

    let _guard = BUILD_LOCK.lock().await;

    // Resolve and materialize the requested version in an isolated worktree.
    let _ = log.send("Preparing UbiHome source…".to_string());
    let reference = opts.repo.resolve(opts.reference.as_deref())?;
    let _ = log.send(format!("Using version: {reference}"));
    let workdir = opts.repo.worktree(&reference, &reference)?;

    let cargo_toml = workdir.join("Cargo.toml");
    let original = std::fs::read_to_string(&cargo_toml)
        .map_err(|e| BuilderError::Source(format!("cannot read Cargo.toml: {e}")))?;

    // Validate the selection against what this version can build.
    let available = platforms::available_components(&cargo_toml)?;
    for c in &selected {
        if !available.contains(c) {
            return Err(BuilderError::Config(format!(
                "config references unknown platform '{c}' for {reference}. Available: {}",
                available.join(", ")
            )));
        }
    }
    let _ = log.send(format!("Selected components: {}", selected.join(", ")));

    // Trim the throwaway worktree's Cargo.toml.
    let trimmed = trim_cargo_toml(&original, &selected)?;
    std::fs::write(&cargo_toml, &trimmed)
        .map_err(|e| BuilderError::Source(format!("cannot write Cargo.toml: {e}")))?;

    let target_label = format!("{}-build", reference);
    let _ = log.send(format!(
        "$ {} build --release --bin ubihome{}",
        if opts.use_cross { "cross" } else { "cargo" },
        opts.target
            .as_ref()
            .map(|t| format!(" --target {t}"))
            .unwrap_or_default()
    ));
    let mut cmd = cargo_build_command(
        &opts.repo,
        &workdir,
        &target_label,
        &opts.target,
        opts.use_cross,
    );
    let child = cmd
        .spawn()
        .map_err(|e| BuilderError::Build(format!("failed to start build: {e}")))?;
    let status = run_streaming(child, &log).await?;
    if !status.success() {
        return Err(BuilderError::Build(format!(
            "cargo build failed with status {status}"
        )));
    }

    let (built, is_windows) = built_binary(&opts.repo, &target_label, &opts.target);
    if !built.is_file() {
        return Err(BuilderError::Build(format!(
            "build reported success but artifact not found at {}",
            built.display()
        )));
    }

    // Copy to the output dir under a release-style, version-tagged name.
    // Include a config-name part so different configs don't collide on filename.
    let target_name = opts.target.clone().unwrap_or_else(targets::host_triple);
    let suffix = targets::artifact_suffix(&target_name);
    let core = match name_part(opts.name.as_deref()) {
        Some(n) => format!("ubihome-{n}-{reference}-{suffix}"),
        None => format!("ubihome-{reference}-{suffix}"),
    };
    let out_name = if is_windows {
        format!("{core}.exe")
    } else {
        core
    };
    std::fs::create_dir_all(&opts.output_dir).map_err(|e| {
        BuilderError::Build(format!(
            "cannot create output dir {}: {e}",
            opts.output_dir.display()
        ))
    })?;
    let dest = opts.output_dir.join(&out_name);
    std::fs::copy(&built, &dest)
        .map_err(|e| BuilderError::Build(format!("cannot copy artifact: {e}")))?;
    let size = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);

    let _ = log.send(format!(
        "Done: {} ({:.1} MB)",
        dest.display(),
        size as f64 / 1_048_576.0
    ));

    Ok(BuildArtifact {
        path: dest,
        size,
        components: selected,
        version: reference,
        target: opts.target.unwrap_or_else(|| "host".into()),
        artifact: suffix,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[package]
name = "ubihome"

[dependencies]
tokio = "1"
ubihome-api = { path = "components/api" }
ubihome-core = { path = "components/core" }
ubihome-mqtt = { path = "components/mqtt" }
ubihome-shell = { path = "components/shell" }
serde = "1"

[workspace]
members = ["components/api", "components/mqtt"]
"#;

    #[test]
    fn trims_to_selected_components_keeping_core_and_others() {
        let out = trim_cargo_toml(SAMPLE, &["mqtt".to_string()]).unwrap();
        assert!(out.contains("ubihome-core"));
        assert!(out.contains("ubihome-mqtt"));
        assert!(!out.contains("ubihome-api"));
        assert!(!out.contains("ubihome-shell"));
        assert!(out.contains("tokio"));
        assert!(out.contains("serde"));
        assert!(out.contains("[workspace]"));
    }

    #[test]
    fn name_part_slugs_and_strips_extension() {
        assert_eq!(name_part(Some("living.yml")).as_deref(), Some("living"));
        assert_eq!(name_part(Some("garden.yaml")).as_deref(), Some("garden"));
        assert_eq!(name_part(Some("my room")).as_deref(), Some("my_room"));
        assert_eq!(name_part(Some("")), None);
        assert_eq!(name_part(None), None);
    }

    #[test]
    fn empty_selection_drops_all_but_core() {
        let out = trim_cargo_toml(SAMPLE, &[]).unwrap();
        assert!(out.contains("ubihome-core"));
        assert!(!out.contains("ubihome-api"));
        assert!(!out.contains("ubihome-mqtt"));
        assert!(!out.contains("ubihome-shell"));
    }
}
