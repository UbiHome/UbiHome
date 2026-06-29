//! Git-backed source management. The builder is fully decoupled from the repo it
//! ships in: it clones the UbiHome repository on demand into a cache directory
//! and materializes each requested version as an isolated `git worktree`. Builds
//! mutate only that throwaway worktree (its `Cargo.toml` is trimmed), never any
//! shared source tree. A worktree is reset to the pristine ref before each use.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::BuilderError;

/// Default public UbiHome repository (clone source).
pub const DEFAULT_REPO_URL: &str = "https://github.com/UbiHome/UbiHome.git";

/// A managed clone of the UbiHome repository plus its working areas, all under
/// one cache root (typically a Docker volume or the user's cache dir):
/// ```text
/// <cache_root>/repo            the clone (all tags)
/// <cache_root>/worktrees/<id>  isolated per-version checkouts
/// <cache_root>/target/<id>     per-version cargo target dirs
/// <cache_root>/cargo           shared CARGO_HOME
/// <cache_root>/bin/<ref>       cached validator binaries
/// ```
#[derive(Debug, Clone)]
pub struct Repo {
    pub url: String,
    pub cache_root: PathBuf,
}

/// Make a ref safe to use as a directory name.
fn sanitize(label: &str) -> String {
    label
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Parse a stable `vMAJOR.MINOR.PATCH` tag (no pre-release suffix) into a
/// comparable tuple. Returns None for pre-releases (`-next`, etc.) or non-tags.
fn parse_stable(tag: &str) -> Option<(u64, u64, u64)> {
    let v = tag.strip_prefix('v')?;
    let parts: Vec<&str> = v.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let a = parts[0].parse().ok()?;
    let b = parts[1].parse().ok()?;
    let c = parts[2].parse().ok()?;
    Some((a, b, c))
}

impl Repo {
    pub fn new(url: impl Into<String>, cache_root: impl Into<PathBuf>) -> Self {
        Self {
            url: url.into(),
            cache_root: cache_root.into(),
        }
    }

    pub fn repo_dir(&self) -> PathBuf {
        self.cache_root.join("repo")
    }
    pub fn cargo_home(&self) -> PathBuf {
        self.cache_root.join("cargo")
    }
    pub fn target_dir(&self, label: &str) -> PathBuf {
        self.cache_root.join("target").join(sanitize(label))
    }
    pub fn validator_bin(&self, reference: &str) -> PathBuf {
        let exe = if cfg!(windows) {
            "ubihome.exe"
        } else {
            "ubihome"
        };
        self.cache_root
            .join("bin")
            .join(sanitize(reference))
            .join(exe)
    }

    /// Run a git command, returning trimmed stdout or an error with stderr.
    fn git(&self, args: &[&str]) -> Result<String, BuilderError> {
        let out = Command::new("git").args(args).output().map_err(|e| {
            BuilderError::Source(format!("failed to run git (is it installed?): {e}"))
        })?;
        if !out.status.success() {
            return Err(BuilderError::Source(format!(
                "git {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    /// Clone the repository if it is not already present.
    pub fn ensure_cloned(&self) -> Result<(), BuilderError> {
        if self.repo_dir().join(".git").exists() {
            return Ok(());
        }
        std::fs::create_dir_all(&self.cache_root)
            .map_err(|e| BuilderError::Source(format!("cannot create cache dir: {e}")))?;
        self.git(&[
            "clone",
            "--no-single-branch",
            &self.url,
            &self.repo_dir().to_string_lossy(),
        ])?;
        Ok(())
    }

    /// Refresh tags/branches from the remote.
    pub fn fetch(&self) -> Result<(), BuilderError> {
        let dir = self.repo_dir();
        let dir = dir.to_string_lossy();
        self.git(&[
            "-C", &dir, "fetch", "--tags", "--prune", "--force", "origin",
        ])?;
        Ok(())
    }

    /// All tags currently known to the clone.
    pub fn tags(&self) -> Result<Vec<String>, BuilderError> {
        let dir = self.repo_dir();
        let out = self.git(&["-C", &dir.to_string_lossy(), "tag"])?;
        Ok(out
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }

    /// Stable version tags only (no pre-releases), newest first.
    pub fn stable_versions(&self) -> Result<Vec<String>, BuilderError> {
        let mut versions: Vec<(String, (u64, u64, u64))> = self
            .tags()?
            .into_iter()
            .filter_map(|t| parse_stable(&t).map(|v| (t, v)))
            .collect();
        versions.sort_by_key(|(_, v)| std::cmp::Reverse(*v));
        Ok(versions.into_iter().map(|(t, _)| t).collect())
    }

    /// The newest stable version tag, if any.
    pub fn latest_stable(&self) -> Result<Option<String>, BuilderError> {
        Ok(self.stable_versions()?.into_iter().next())
    }

    /// Does a ref (tag/branch/commit) resolve in the clone?
    fn ref_exists(&self, reference: &str) -> bool {
        let dir = self.repo_dir();
        self.git(&[
            "-C",
            &dir.to_string_lossy(),
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{reference}^{{commit}}"),
        ])
        .map(|s| !s.is_empty())
        .unwrap_or(false)
    }

    /// Resolve which ref a build should use. `None` = latest stable tag.
    /// Fetches from the remote to stay current / to find a requested ref.
    pub fn resolve(&self, requested: Option<&str>) -> Result<String, BuilderError> {
        self.ensure_cloned()?;
        match requested {
            Some(r) => {
                if !self.ref_exists(r) {
                    self.fetch()?;
                }
                if !self.ref_exists(r) {
                    return Err(BuilderError::Source(format!(
                        "version/ref '{r}' not found in {}",
                        self.url
                    )));
                }
                Ok(r.to_string())
            }
            None => {
                self.fetch()?;
                self.latest_stable()?.ok_or_else(|| {
                    BuilderError::Source("no stable version tags found in repository".into())
                })
            }
        }
    }

    /// Ensure an isolated worktree exists for `reference`, reset to its pristine
    /// state, and return its path. `label` distinguishes purposes (e.g. build vs
    /// validate) so they don't fight over the same checkout/cache.
    pub fn worktree(&self, label: &str, reference: &str) -> Result<PathBuf, BuilderError> {
        let repo = self.repo_dir();
        let repo = repo.to_string_lossy();
        let wt = self.cache_root.join("worktrees").join(sanitize(label));
        std::fs::create_dir_all(wt.parent().unwrap())
            .map_err(|e| BuilderError::Source(format!("cannot create worktrees dir: {e}")))?;
        let wt_str = wt.to_string_lossy().to_string();

        if wt.join("Cargo.toml").exists() {
            // Reuse: reset to the pristine ref (drops any prior Cargo.toml trim).
            self.git(&["-C", &wt_str, "reset", "--hard", reference])?;
            self.git(&["-C", &wt_str, "clean", "-fd"])?;
        } else {
            // Create fresh; prune any stale registration first.
            let _ = self.git(&["-C", &repo, "worktree", "prune"]);
            self.git(&[
                "-C", &repo, "worktree", "add", "--force", "--detach", &wt_str, reference,
            ])?;
        }
        Ok(wt)
    }
}

/// Default cache root for the CLI when not otherwise configured:
/// `$BUILDER_WORK`, else `$HOME/.cache/ubihome-builder`, else `./.ubihome-builder`.
pub fn default_cache_root() -> PathBuf {
    if let Ok(w) = std::env::var("BUILDER_WORK") {
        return PathBuf::from(w);
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return Path::new(&home).join(".cache").join("ubihome-builder");
        }
    }
    PathBuf::from("./.ubihome-builder")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_parse_excludes_prereleases() {
        assert_eq!(parse_stable("v0.14.0"), Some((0, 14, 0)));
        assert_eq!(parse_stable("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_stable("v0.15.0-next.5"), None);
        assert_eq!(parse_stable("0.14.0"), None); // requires leading v
        assert_eq!(parse_stable("v0.14"), None);
    }

    #[test]
    fn sanitize_makes_safe_dir_names() {
        assert_eq!(sanitize("v0.14.0"), "v0.14.0");
        assert_eq!(sanitize("origin/main"), "origin_main");
        assert_eq!(sanitize("feature/x y"), "feature_x_y");
    }
}
