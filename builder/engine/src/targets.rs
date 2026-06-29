//! Host detection and the set of compile targets feasible from the current host.
//!
//! A Linux Docker container can only emit Linux (host arch) and cross-compiled
//! ARM binaries. macOS/Windows binaries must be built on a native host. So the
//! feasible set is: always the host triple, plus any cross targets we know how
//! to reach from this host (ARM musl from Linux, via `cross` + `Cross.toml`).

use std::process::Command;

/// A compile target the builder can produce on this host.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Target {
    /// Rust target triple, e.g. `aarch64-apple-darwin`.
    pub triple: String,
    /// Human label, e.g. `macOS (Apple Silicon)`.
    pub label: String,
    /// Release-style artifact suffix, e.g. `macos-aarch64`.
    pub artifact: String,
    /// Whether this target requires the `cross` tool (vs plain `cargo`).
    pub needs_cross: bool,
    /// True for the native host target.
    pub is_host: bool,
}

/// The host's own target triple, as reported by `rustc -vV`.
pub fn host_triple() -> String {
    let out = Command::new("rustc").arg("-vV").output();
    if let Ok(out) = out {
        if let Ok(text) = String::from_utf8(out.stdout) {
            for line in text.lines() {
                if let Some(rest) = line.strip_prefix("host: ") {
                    return rest.trim().to_string();
                }
            }
        }
    }
    // Fall back to the triple this engine was compiled for.
    std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string())
}

/// Map a target triple to a release-style `<os>-<arch>` artifact suffix,
/// matching the naming used by `.github/workflows/release.yml`.
pub fn artifact_suffix(triple: &str) -> String {
    match triple {
        "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl" => "linux-x86_64".into(),
        "aarch64-unknown-linux-gnu" | "aarch64-unknown-linux-musl" => "linux-aarch64".into(),
        "armv7-unknown-linux-musleabi" | "armv7-unknown-linux-gnueabihf" => "linux-armv7".into(),
        "arm-unknown-linux-musleabi" | "arm-unknown-linux-gnueabi" => "linux-arm".into(),
        "x86_64-apple-darwin" => "macos-x86_64".into(),
        "aarch64-apple-darwin" => "macos-aarch64".into(),
        "x86_64-pc-windows-msvc" | "x86_64-pc-windows-gnu" => "windows-x86_64".into(),
        other => other.to_string(),
    }
}

fn label_for(triple: &str) -> String {
    match triple {
        t if t.contains("apple-darwin") && t.starts_with("aarch64") => {
            "macOS (Apple Silicon)".into()
        }
        t if t.contains("apple-darwin") => "macOS (Intel)".into(),
        t if t.contains("windows") => "Windows (x86_64)".into(),
        "armv7-unknown-linux-musleabi" => "Linux ARMv7 (Raspberry Pi 3/4)".into(),
        "arm-unknown-linux-musleabi" => "Linux ARM (Raspberry Pi Zero/1)".into(),
        t if t.contains("linux") && t.starts_with("aarch64") => "Linux (ARM64)".into(),
        t if t.contains("linux") => "Linux (x86_64)".into(),
        other => other.to_string(),
    }
}

/// Is a given target triple installed for the current rustup toolchain?
fn rustup_target_installed(triple: &str) -> bool {
    Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().any(|l| l.trim() == triple))
        .unwrap_or(false)
}

fn have_cross() -> bool {
    Command::new("cross")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Compute the targets that can actually be built on this host right now.
///
/// Always includes the host target (built with plain `cargo`). On Linux, also
/// offers the ARM musl targets when reachable: either an installed rustup target
/// or via the `cross` tool (which uses Docker + `Cross.toml`).
pub fn feasible_targets() -> Vec<Target> {
    let host = host_triple();
    let mut targets = vec![Target {
        triple: host.clone(),
        label: label_for(&host),
        artifact: artifact_suffix(&host),
        needs_cross: false,
        is_host: true,
    }];

    if host.contains("linux") {
        let cross_available = have_cross();
        for triple in ["armv7-unknown-linux-musleabi", "arm-unknown-linux-musleabi"] {
            let installed = rustup_target_installed(triple);
            if installed || cross_available {
                targets.push(Target {
                    triple: triple.to_string(),
                    label: label_for(triple),
                    artifact: artifact_suffix(triple),
                    // Prefer cross for the musl ARM targets (handles the ALSA
                    // cross-build defined in Cross.toml); fall back to cargo if
                    // the target is locally installed but cross is absent.
                    needs_cross: cross_available && !installed,
                    is_host: false,
                });
            }
        }
    }

    targets
}
