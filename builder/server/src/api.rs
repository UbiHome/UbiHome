//! REST + WebSocket handlers for the dashboard. All real work is delegated to
//! the engine; these functions are thin adapters over it.

use std::path::PathBuf;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::sync::broadcast;
use ubihome_builder_engine as engine;

use crate::AppState;

/// Map an engine error to an HTTP error response.
fn err(status: StatusCode, msg: impl ToString) -> Response {
    (
        status,
        Json(serde_json::json!({ "error": msg.to_string() })),
    )
        .into_response()
}

fn engine_err(e: engine::BuilderError) -> Response {
    let status = match e {
        engine::BuilderError::Config(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    err(status, e)
}

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

pub async fn targets() -> Json<Vec<engine::Target>> {
    Json(engine::feasible_targets())
}

/// Optional `?ref=<tag|branch|sha>` selector used by validate/build.
#[derive(Deserialize, Default)]
pub struct RefQuery {
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

/// List buildable UbiHome versions (stable tags, newest first).
pub async fn versions(State(st): State<AppState>) -> Response {
    let repo = st.repo.clone();
    // Cloning + fetching touches the network and git; do it off the async runtime.
    let result = tokio::task::spawn_blocking(move || {
        repo.ensure_cloned()?;
        repo.fetch()?;
        repo.stable_versions()
    })
    .await;
    match result {
        Ok(Ok(versions)) => {
            let latest = versions.first().cloned();
            Json(serde_json::json!({ "latest": latest, "versions": versions })).into_response()
        }
        Ok(Err(e)) => engine_err(e),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

pub async fn list_configs(State(st): State<AppState>) -> Response {
    match st.store.list_configs() {
        Ok(list) => Json(list).into_response(),
        Err(e) => engine_err(e),
    }
}

#[derive(Deserialize)]
pub struct CreateConfig {
    pub name: String,
    #[serde(default)]
    pub content: String,
}

pub async fn create_config(State(st): State<AppState>, Json(body): Json<CreateConfig>) -> Response {
    let content = if body.content.is_empty() {
        default_config(&body.name)
    } else {
        body.content
    };
    match st.store.write_config(&body.name, &content) {
        Ok(()) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "name": body.name })),
        )
            .into_response(),
        Err(e) => engine_err(e),
    }
}

#[derive(Serialize)]
pub struct ConfigDetail {
    name: String,
    content: String,
    components: Vec<String>,
}

pub async fn get_config(State(st): State<AppState>, Path(name): Path<String>) -> Response {
    match st.store.read_config(&name) {
        Ok(content) => Json(ConfigDetail {
            components: engine::detect_platforms(&content),
            name,
            content,
        })
        .into_response(),
        Err(e) => err(StatusCode::NOT_FOUND, e),
    }
}

#[derive(Deserialize)]
pub struct UpdateConfig {
    pub content: String,
}

pub async fn update_config(
    State(st): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<UpdateConfig>,
) -> Response {
    match st.store.write_config(&name, &body.content) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => engine_err(e),
    }
}

pub async fn delete_config(State(st): State<AppState>, Path(name): Path<String>) -> Response {
    match st.store.delete_config(&name) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => engine_err(e),
    }
}

#[derive(Deserialize)]
pub struct RenameBody {
    pub to: String,
}

pub async fn rename_config(
    State(st): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<RenameBody>,
) -> Response {
    match st.store.rename_config(&name, &body.to) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => engine_err(e),
    }
}

pub async fn duplicate_config(
    State(st): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<RenameBody>,
) -> Response {
    match st.store.duplicate_config(&name, &body.to) {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(e) => engine_err(e),
    }
}

pub async fn validate_config(
    State(st): State<AppState>,
    Path(name): Path<String>,
    Query(q): Query<RefQuery>,
) -> Response {
    let content = match st.store.read_config(&name) {
        Ok(c) => c,
        Err(e) => return err(StatusCode::NOT_FOUND, e),
    };
    match engine::validate_config(&st.repo, q.reference.as_deref(), &content).await {
        Ok(result) => Json(result).into_response(),
        Err(e) => engine_err(e),
    }
}

pub async fn detect_config(State(st): State<AppState>, Path(name): Path<String>) -> Response {
    match st.store.read_config(&name) {
        Ok(content) => Json(serde_json::json!({
            "components": engine::detect_platforms(&content)
        }))
        .into_response(),
        Err(e) => err(StatusCode::NOT_FOUND, e),
    }
}

#[derive(Deserialize)]
pub struct BuildBody {
    /// Target triple. None/empty = native host.
    #[serde(default)]
    pub target: Option<String>,
    /// Version/ref to build. None/empty = latest stable tag.
    #[serde(default, rename = "ref")]
    pub reference: Option<String>,
}

/// Start a build: create a history entry, spawn the build task, and return the
/// build id. The client then connects to the WebSocket to stream logs.
pub async fn start_build(
    State(st): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<BuildBody>,
) -> Response {
    let content = match st.store.read_config(&name) {
        Ok(c) => c,
        Err(e) => return err(StatusCode::NOT_FOUND, e),
    };
    let components = engine::detect_platforms(&content);
    if components.is_empty() {
        return err(StatusCode::BAD_REQUEST, "config has no components to build");
    }

    let target = body.target.filter(|t| !t.is_empty());
    let reference = body.reference.filter(|t| !t.is_empty());
    let use_cross = target
        .as_ref()
        .map(|t| {
            engine::feasible_targets()
                .iter()
                .any(|ft| &ft.triple == t && ft.needs_cross)
        })
        .unwrap_or(false);

    let target_label = target.clone().unwrap_or_else(|| "host".into());
    let version_label = reference.clone().unwrap_or_else(|| "latest".into());
    let id = match st
        .store
        .start_build(&name, &version_label, &target_label, &components)
    {
        Ok(id) => id,
        Err(e) => return engine_err(e),
    };

    // Broadcast channel for live log subscribers.
    let (btx, _) = broadcast::channel::<String>(1024);
    st.live.lock().await.insert(id, btx.clone());

    // Each build writes to its own subdir so artifacts never overwrite each
    // other and history downloads stay correct.
    let opts = engine::BuildOptions {
        repo: st.repo.clone(),
        reference,
        config: content,
        name: Some(name.clone()),
        output_dir: st.store.output_dir().join(id.to_string()),
        target,
        use_cross,
    };

    let store = st.store.clone();
    let live = st.live.clone();
    let log_path = st.store.log_path(id);

    tokio::spawn(async move {
        run_build_job(id, opts, store, live, btx, log_path).await;
    });

    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({ "build_id": id })),
    )
        .into_response()
}

/// Drive one build: bridge engine log lines to the broadcast channel and a log
/// file, then record the outcome in history.
async fn run_build_job(
    id: u64,
    opts: engine::BuildOptions,
    store: engine::Store,
    live: crate::LiveLogs,
    btx: broadcast::Sender<String>,
    log_path: PathBuf,
) {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    // Forward engine logs to the broadcast channel and the on-disk log file.
    let btx_fwd = btx.clone();
    let forward = tokio::spawn(async move {
        let mut file = tokio::fs::File::create(&log_path).await.ok();
        while let Some(line) = rx.recv().await {
            let _ = btx_fwd.send(line.clone());
            if let Some(f) = file.as_mut() {
                let _ = f.write_all(line.as_bytes()).await;
                let _ = f.write_all(b"\n").await;
            }
        }
    });

    let result = engine::build(opts, tx).await;
    let _ = forward.await;

    match &result {
        Ok(artifact) => {
            let name = artifact
                .path
                .file_name()
                .map(|s| s.to_string_lossy().into_owned());
            let _ = store.finish_build(
                id,
                "success",
                Some(&artifact.version),
                name.as_deref(),
                artifact.size,
            );
            let _ = btx.send("__BUILD_DONE__:success".into());
        }
        Err(e) => {
            let _ = btx.send(format!("ERROR: {e}"));
            let _ = store.finish_build(id, "failed", None, None, 0);
            let _ = btx.send("__BUILD_DONE__:failed".into());
        }
    }

    // Drop the live channel once finished.
    live.lock().await.remove(&id);
}

pub async fn list_builds(State(st): State<AppState>) -> Response {
    match st.store.list_builds() {
        Ok(list) => Json(list).into_response(),
        Err(e) => engine_err(e),
    }
}

pub async fn get_build(State(st): State<AppState>, Path(id): Path<u64>) -> Response {
    match st.store.get_build(id) {
        Ok(Some(r)) => Json(r).into_response(),
        Ok(None) => err(StatusCode::NOT_FOUND, "no such build"),
        Err(e) => engine_err(e),
    }
}

pub async fn get_build_log(State(st): State<AppState>, Path(id): Path<u64>) -> Response {
    match tokio::fs::read_to_string(st.store.log_path(id)).await {
        Ok(text) => ([(header::CONTENT_TYPE, "text/plain")], text).into_response(),
        Err(_) => err(StatusCode::NOT_FOUND, "no log for build"),
    }
}

/// Download a finished build's artifact.
pub async fn download_artifact(State(st): State<AppState>, Path(id): Path<u64>) -> Response {
    let record = match st.store.get_build(id) {
        Ok(Some(r)) => r,
        Ok(None) => return err(StatusCode::NOT_FOUND, "no such build"),
        Err(e) => return engine_err(e),
    };
    let Some(name) = record.artifact else {
        return err(StatusCode::NOT_FOUND, "build has no artifact");
    };
    // Artifacts live in a per-build subdir (see start_build).
    let path = st.store.output_dir().join(id.to_string()).join(&name);
    match tokio::fs::read(&path).await {
        Ok(bytes) => (
            [
                (header::CONTENT_TYPE, "application/octet-stream".to_string()),
                (
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{name}\""),
                ),
            ],
            bytes,
        )
            .into_response(),
        Err(_) => err(StatusCode::NOT_FOUND, "artifact file missing"),
    }
}

/// WebSocket: stream a build's logs live. If the build already finished, replay
/// the captured log file and close.
pub async fn build_log_ws(
    ws: WebSocketUpgrade,
    State(st): State<AppState>,
    Path(id): Path<u64>,
) -> Response {
    ws.on_upgrade(move |socket| stream_logs(socket, st, id))
}

async fn stream_logs(mut socket: WebSocket, st: AppState, id: u64) {
    // Subscribe to live logs if the build is still running.
    let rx = st.live.lock().await.get(&id).map(|tx| tx.subscribe());

    match rx {
        Some(mut rx) => loop {
            match rx.recv().await {
                Ok(line) => {
                    if let Some(rest) = line.strip_prefix("__BUILD_DONE__:") {
                        let _ = socket.send(Message::Text(format!("[build {rest}]"))).await;
                        break;
                    }
                    if socket.send(Message::Text(line)).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        },
        None => {
            // Build finished (or never streamed): replay the log file.
            if let Ok(text) = tokio::fs::read_to_string(st.store.log_path(id)).await {
                for line in text.lines() {
                    if socket.send(Message::Text(line.to_string())).await.is_err() {
                        break;
                    }
                }
            }
            let _ = socket.send(Message::Text("[build finished]".into())).await;
        }
    }
    let _ = socket.close().await;
}

/// A minimal starter config for newly created entries.
fn default_config(name: &str) -> String {
    format!(
        r#"ubihome:
  name: "{name}"

logger:
  level: info

# Add platform components below, e.g.:
# api:
# mqtt:
#   broker: localhost
"#
    )
}
