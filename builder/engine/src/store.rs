//! Persistence for the dashboard: multiple named configs on disk, plus a build
//! history with per-build logs and artifacts. Intentionally simple (plain files
//! + a JSON index) so the runtime image needs no database.
//!
//! Layout under the data root:
//! ```text
//! <root>/configs/<name>.yml      user configs
//! <root>/output/<artifact>       built binaries
//! <root>/logs/<build-id>.log     captured build logs
//! <root>/builds.json             build history index
//! ```

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::platforms;
use crate::BuilderError;

/// Metadata about a stored config.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConfigInfo {
    pub name: String,
    pub components: Vec<String>,
    pub size: u64,
}

/// One entry in the build history.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BuildRecord {
    pub id: u64,
    pub config: String,
    /// Requested version/ref, then the resolved version once known.
    #[serde(default)]
    pub version: String,
    pub target: String,
    pub components: Vec<String>,
    /// "success" | "failed" | "running".
    pub status: String,
    pub size: u64,
    pub created_at: u64,
    pub artifact: Option<String>,
    pub log_file: Option<String>,
}

/// File-backed store rooted at a data directory (typically a Docker volume).
#[derive(Debug, Clone)]
pub struct Store {
    root: PathBuf,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Reject names that could escape the configs dir or are otherwise unsafe.
fn safe_name(name: &str) -> Result<(), BuilderError> {
    if name.is_empty()
        || name.len() > 128
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        || name.starts_with('.')
    {
        return Err(BuilderError::Config(format!(
            "invalid config name: {name:?}"
        )));
    }
    Ok(())
}

impl Store {
    /// Open (creating directories as needed) a store at `root`.
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, BuilderError> {
        let root = root.into();
        for sub in ["configs", "output", "logs"] {
            std::fs::create_dir_all(root.join(sub))
                .map_err(|e| BuilderError::Source(format!("cannot create {sub} dir: {e}")))?;
        }
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn configs_dir(&self) -> PathBuf {
        self.root.join("configs")
    }
    pub fn output_dir(&self) -> PathBuf {
        self.root.join("output")
    }
    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }

    fn config_path(&self, name: &str) -> Result<PathBuf, BuilderError> {
        safe_name(name)?;
        Ok(self.configs_dir().join(name))
    }

    /// List all configs with their detected components.
    pub fn list_configs(&self) -> Result<Vec<ConfigInfo>, BuilderError> {
        let mut out = Vec::new();
        let entries = std::fs::read_dir(self.configs_dir())
            .map_err(|e| BuilderError::Source(format!("cannot read configs dir: {e}")))?;
        for entry in entries.flatten() {
            if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            out.push(ConfigInfo {
                name,
                components: platforms::detect_platforms(&content),
                size,
            });
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }

    pub fn read_config(&self, name: &str) -> Result<String, BuilderError> {
        let path = self.config_path(name)?;
        std::fs::read_to_string(&path)
            .map_err(|e| BuilderError::Config(format!("cannot read config {name}: {e}")))
    }

    pub fn write_config(&self, name: &str, content: &str) -> Result<(), BuilderError> {
        let path = self.config_path(name)?;
        std::fs::write(&path, content)
            .map_err(|e| BuilderError::Config(format!("cannot write config {name}: {e}")))
    }

    pub fn delete_config(&self, name: &str) -> Result<(), BuilderError> {
        let path = self.config_path(name)?;
        std::fs::remove_file(&path)
            .map_err(|e| BuilderError::Config(format!("cannot delete config {name}: {e}")))
    }

    pub fn rename_config(&self, from: &str, to: &str) -> Result<(), BuilderError> {
        let src = self.config_path(from)?;
        let dst = self.config_path(to)?;
        std::fs::rename(&src, &dst)
            .map_err(|e| BuilderError::Config(format!("cannot rename config: {e}")))
    }

    pub fn duplicate_config(&self, from: &str, to: &str) -> Result<(), BuilderError> {
        let src = self.config_path(from)?;
        let dst = self.config_path(to)?;
        std::fs::copy(&src, &dst)
            .map_err(|e| BuilderError::Config(format!("cannot duplicate config: {e}")))?;
        Ok(())
    }

    fn builds_index(&self) -> PathBuf {
        self.root.join("builds.json")
    }

    /// Read the build history (newest first).
    pub fn list_builds(&self) -> Result<Vec<BuildRecord>, BuilderError> {
        let path = self.builds_index();
        if !path.is_file() {
            return Ok(Vec::new());
        }
        let text = std::fs::read_to_string(&path)
            .map_err(|e| BuilderError::Source(format!("cannot read builds.json: {e}")))?;
        let mut records: Vec<BuildRecord> = serde_json::from_str(&text).unwrap_or_default();
        records.sort_by_key(|r| std::cmp::Reverse(r.id));
        Ok(records)
    }

    pub fn get_build(&self, id: u64) -> Result<Option<BuildRecord>, BuilderError> {
        Ok(self.list_builds()?.into_iter().find(|r| r.id == id))
    }

    fn write_builds(&self, records: &[BuildRecord]) -> Result<(), BuilderError> {
        let text = serde_json::to_string_pretty(records)
            .map_err(|e| BuilderError::Source(format!("cannot serialize builds: {e}")))?;
        std::fs::write(self.builds_index(), text)
            .map_err(|e| BuilderError::Source(format!("cannot write builds.json: {e}")))
    }

    /// Create a new history entry in the "running" state and return its id.
    pub fn start_build(
        &self,
        config: &str,
        version: &str,
        target: &str,
        components: &[String],
    ) -> Result<u64, BuilderError> {
        let mut records = self.list_builds()?;
        let id = records.iter().map(|r| r.id).max().unwrap_or(0) + 1;
        records.push(BuildRecord {
            id,
            config: config.to_string(),
            version: version.to_string(),
            target: target.to_string(),
            components: components.to_vec(),
            status: "running".into(),
            size: 0,
            created_at: now_secs(),
            artifact: None,
            log_file: Some(format!("{id}.log")),
        });
        self.write_builds(&records)?;
        Ok(id)
    }

    /// Mark a build finished, recording status, resolved version, artifact and size.
    pub fn finish_build(
        &self,
        id: u64,
        status: &str,
        version: Option<&str>,
        artifact: Option<&str>,
        size: u64,
    ) -> Result<(), BuilderError> {
        let mut records = self.list_builds()?;
        if let Some(r) = records.iter_mut().find(|r| r.id == id) {
            r.status = status.to_string();
            r.artifact = artifact.map(|s| s.to_string());
            r.size = size;
            if let Some(v) = version {
                r.version = v.to_string();
            }
        }
        self.write_builds(&records)
    }

    pub fn log_path(&self, id: u64) -> PathBuf {
        self.logs_dir().join(format!("{id}.log"))
    }
}
