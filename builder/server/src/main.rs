//! `ubihome-builder-server` — the web dashboard backend.
//!
//! Wraps the shared engine with a REST API + WebSocket build-log streaming, and
//! serves the embedded Angular SPA. This is the binary that runs inside Docker.

mod api;
mod assets;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use clap::Parser;
use tokio::sync::{broadcast, Mutex};
use tower_http::cors::CorsLayer;
use ubihome_builder_engine::{Repo, Store, DEFAULT_REPO_URL};

/// Live log channels for in-progress builds, keyed by build id.
pub type LiveLogs = Arc<Mutex<HashMap<u64, broadcast::Sender<String>>>>;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub store: Store,
    pub repo: Repo,
    pub live: LiveLogs,
}

#[derive(Parser)]
#[command(
    name = "ubihome-builder-server",
    about = "UbiHome builder web dashboard"
)]
struct Args {
    /// Address to bind.
    #[arg(long, env = "BUILDER_BIND", default_value = "0.0.0.0:8080")]
    bind: SocketAddr,
    /// Data directory (configs, output, logs, history).
    #[arg(long, env = "BUILDER_DATA", default_value = "./data")]
    data: PathBuf,
    /// Cache dir for the UbiHome clone, worktrees and cargo cache.
    #[arg(long, env = "BUILDER_WORK", default_value = "./cache")]
    work: PathBuf,
    /// UbiHome git repository to build from (URL or local path).
    #[arg(long, env = "BUILDER_REPO_URL", default_value = DEFAULT_REPO_URL)]
    repo_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = Args::parse();
    let store = Store::open(&args.data)?;
    let repo = Repo::new(args.repo_url, &args.work);
    tracing::info!("data dir:   {}", store.root().display());
    tracing::info!("cache dir:  {}", args.work.display());
    tracing::info!("ubihome repo: {}", repo.url);

    let state = AppState {
        store,
        repo,
        live: Arc::new(Mutex::new(HashMap::new())),
    };

    let api = Router::new()
        .route("/health", get(api::health))
        .route("/targets", get(api::targets))
        .route("/versions", get(api::versions))
        .route("/configs", get(api::list_configs).post(api::create_config))
        .route(
            "/configs/:name",
            get(api::get_config)
                .put(api::update_config)
                .delete(api::delete_config),
        )
        .route("/configs/:name/validate", post(api::validate_config))
        .route("/configs/:name/detect", post(api::detect_config))
        .route("/configs/:name/rename", post(api::rename_config))
        .route("/configs/:name/duplicate", post(api::duplicate_config))
        .route("/configs/:name/build", post(api::start_build))
        .route("/builds", get(api::list_builds))
        .route("/builds/:id", get(api::get_build))
        .route("/builds/:id/log", get(api::get_build_log))
        .route("/builds/:id/logs", get(api::build_log_ws))
        .route("/builds/:id/artifact", get(api::download_artifact))
        .with_state(state.clone());

    let app = Router::new()
        .nest("/api", api)
        .fallback(assets::static_handler)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(args.bind).await?;
    tracing::info!(
        "UbiHome Builder dashboard listening on http://{}",
        args.bind
    );
    axum::serve(listener, app).await?;
    Ok(())
}
