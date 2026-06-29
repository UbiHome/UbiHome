//! `ubihome-builder` — lean CLI front-end to the builder engine.
//!
//! No web dependencies (no axum, no embedded SPA), so a mac/Windows user can run
//! it natively to produce a host binary. It clones the UbiHome repo on demand
//! and builds any tagged version in isolation — fully decoupled from any local
//! UbiHome checkout. Shares all logic with the server via the engine crate.

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use ubihome_builder_engine as engine;

#[derive(Parser)]
#[command(
    name = "ubihome-builder",
    about = "Build a slim UbiHome binary from a config.yml"
)]
struct Cli {
    /// UbiHome git repository to build from (URL or local path).
    #[arg(long, env = "BUILDER_REPO_URL", default_value = engine::DEFAULT_REPO_URL, global = true)]
    repo_url: String,
    /// Cache dir for the clone, worktrees and cargo cache.
    #[arg(long, env = "BUILDER_WORK", global = true)]
    work: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

impl Cli {
    fn repo(&self) -> engine::Repo {
        let root = self.work.clone().unwrap_or_else(engine::default_cache_root);
        engine::Repo::new(self.repo_url.clone(), root)
    }
}

#[derive(Subcommand)]
enum Command {
    /// Print the platform components a config references.
    Detect {
        #[arg(short, long, default_value = "config.yml")]
        config: PathBuf,
    },
    /// List the compile targets feasible on this host.
    Targets,
    /// List buildable UbiHome versions (stable tags, newest first).
    Versions,
    /// Validate a config using the real UbiHome validator.
    Validate {
        #[arg(short, long, default_value = "config.yml")]
        config: PathBuf,
        /// Version/ref to validate against (default: latest stable tag).
        #[arg(short = 'r', long)]
        r#ref: Option<String>,
    },
    /// Build a slim binary containing only the components the config uses.
    Build {
        #[arg(short, long, default_value = "config.yml")]
        config: PathBuf,
        /// Output directory for the binary.
        #[arg(short, long, default_value = "./output")]
        output: PathBuf,
        /// Version/ref to build (default: latest stable tag).
        #[arg(short = 'r', long)]
        r#ref: Option<String>,
        /// Target triple (defaults to the native host).
        #[arg(short, long)]
        target: Option<String>,
        /// Use `cross` instead of `cargo` (for ARM musl targets).
        #[arg(long)]
        cross: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Command::Detect { config } => {
            let content = std::fs::read_to_string(config)?;
            let platforms = engine::detect_platforms(&content);
            if platforms.is_empty() {
                println!("No platform components found in {}", config.display());
            } else {
                println!("Components in {}:", config.display());
                for p in platforms {
                    println!("  - {p}");
                }
            }
        }
        Command::Targets => {
            println!("Feasible targets on this host:");
            for t in engine::feasible_targets() {
                let host = if t.is_host { " (host)" } else { "" };
                let cross = if t.needs_cross { " [cross]" } else { "" };
                println!("  - {:<32} {}{host}{cross}", t.triple, t.label);
            }
        }
        Command::Versions => {
            let repo = cli.repo();
            repo.ensure_cloned()?;
            repo.fetch()?;
            let versions = repo.stable_versions()?;
            let latest = versions.first().cloned();
            println!("Buildable versions ({}):", repo.url);
            for v in &versions {
                let tag = if Some(v) == latest.as_ref() {
                    "  (latest)"
                } else {
                    ""
                };
                println!("  - {v}{tag}");
            }
        }
        Command::Validate { config, r#ref } => {
            let content = std::fs::read_to_string(config)?;
            let repo = cli.repo();
            let result = engine::validate_config(&repo, r#ref.as_deref(), &content).await?;
            if !result.output.is_empty() {
                println!("{}", result.output);
            }
            if result.ok {
                println!("✔ config is valid (against {})", result.version);
            } else {
                anyhow::bail!("config is invalid (against {})", result.version);
            }
        }
        Command::Build {
            config,
            output,
            r#ref,
            target,
            cross,
        } => {
            let content = std::fs::read_to_string(config)?;
            let use_cross = *cross
                || target
                    .as_ref()
                    .map(|t| {
                        engine::feasible_targets()
                            .iter()
                            .any(|ft| &ft.triple == t && ft.needs_cross)
                    })
                    .unwrap_or(false);

            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
            let printer = tokio::spawn(async move {
                while let Some(line) = rx.recv().await {
                    println!("{line}");
                }
            });

            let opts = engine::BuildOptions {
                repo: cli.repo(),
                reference: r#ref.clone(),
                config: content,
                name: config.file_stem().map(|s| s.to_string_lossy().into_owned()),
                output_dir: output.clone(),
                target: target.clone(),
                use_cross,
            };
            let result = engine::build(opts, tx).await;
            let _ = printer.await;

            let artifact = result?;
            println!(
                "\n✔ Built {} ({:.1} MB) — version {}, components: {}",
                artifact.path.display(),
                artifact.size as f64 / 1_048_576.0,
                artifact.version,
                artifact.components.join(", ")
            );
        }
    }
    Ok(())
}
