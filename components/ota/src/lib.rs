use std::{
    collections::HashMap,
    future::Future,
    net::{IpAddr, SocketAddr},
    pin::Pin,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    body::Bytes,
    extract::{ConnectInfo, DefaultBodyLimit, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use log::{debug, error, info, warn};
use rand::RngCore;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use tokio::{
    net::TcpSocket,
    sync::{
        broadcast::{Receiver, Sender},
        Mutex,
    },
};
use ubihome_core::{
    config_template, internal::sensors::UbiComponent, ChangedMessage, Module, NoConfig,
    PublishedMessage,
};

/// How long an issued challenge nonce stays valid, and thus replayable.
const NONCE_TTL: Duration = Duration::from_secs(30);
/// A whole executable is always well over this; guards against obviously bogus uploads
/// before spending time on the SHA-256 pass.
const MIN_BINARY_SIZE: usize = 1_000_000;
const MAX_FAILED_ATTEMPTS: u32 = 5;
const LOCKOUT_DURATION: Duration = Duration::from_secs(60);

const fn default_port() -> u16 {
    3232
}

const fn default_max_binary_size() -> u64 {
    200_000_000
}

#[derive(Clone, Deserialize, Debug, Validate)]
#[serde(deny_unknown_fields)]
pub struct OtaConfig {
    #[serde(default = "default_port")]
    #[garde(range(min = 1, max = 65535))]
    pub port: u16,

    #[garde(ascii, length(min = 32, max = 256))]
    pub password: String,

    #[serde(default = "default_max_binary_size")]
    #[garde(range(min = 1_000_000, max = 500_000_000))]
    pub max_binary_size: u64,
}

config_template!(
    ota, OtaConfig, NoConfig, NoConfig, NoConfig, NoConfig, NoConfig, NoConfig, NoConfig
);

struct FailureRecord {
    count: u32,
    window_start: Instant,
    locked_until: Option<Instant>,
}

struct AppState {
    config: CoreConfig,
    // Single-use, short-lived challenges: value is when the nonce was issued.
    nonces: Mutex<HashMap<String, Instant>>,
    failed_attempts: Mutex<HashMap<IpAddr, FailureRecord>>,
}

#[derive(Serialize)]
struct ChallengeResponse {
    nonce: String,
}

fn generate_nonce() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

async fn handle_challenge(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let nonce = generate_nonce();
    let now = Instant::now();

    let mut nonces = state.nonces.lock().await;
    nonces.retain(|_, issued_at| now.duration_since(*issued_at) < NONCE_TTL);
    nonces.insert(nonce.clone(), now);

    Json(ChallengeResponse { nonce })
}

async fn is_locked_out(state: &AppState, ip: IpAddr) -> bool {
    let attempts = state.failed_attempts.lock().await;
    match attempts.get(&ip).and_then(|record| record.locked_until) {
        Some(locked_until) => Instant::now() < locked_until,
        None => false,
    }
}

async fn record_auth_failure(state: &AppState, ip: IpAddr) {
    let mut attempts = state.failed_attempts.lock().await;
    let now = Instant::now();
    let record = attempts.entry(ip).or_insert(FailureRecord {
        count: 0,
        window_start: now,
        locked_until: None,
    });

    if now.duration_since(record.window_start) > LOCKOUT_DURATION {
        record.count = 0;
        record.window_start = now;
        record.locked_until = None;
    }

    record.count += 1;
    if record.count >= MAX_FAILED_ATTEMPTS {
        record.locked_until = Some(now + LOCKOUT_DURATION);
    }
}

async fn record_auth_success(state: &AppState, ip: IpAddr) {
    state.failed_attempts.lock().await.remove(&ip);
}

/// Verifies the ESPHome-style nonce/cnonce challenge-response before letting a request
/// reach `handle_upload` - the password itself never has to be sent by the client.
async fn require_valid_nonce(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    req: Request,
    next: Next,
) -> Response {
    let ip = addr.ip();

    if is_locked_out(&state, ip).await {
        warn!("OTA: rejecting {ip}, locked out after repeated auth failures");
        return (StatusCode::TOO_MANY_REQUESTS, "too many failed attempts").into_response();
    }

    let headers_present = (|| {
        let nonce = headers.get("x-ota-nonce")?.to_str().ok()?.to_string();
        let cnonce = headers.get("x-ota-cnonce")?.to_str().ok()?.to_string();
        let auth = headers.get("x-ota-auth")?.to_str().ok()?.to_string();
        Some((nonce, cnonce, auth))
    })();

    let Some((nonce, cnonce, auth)) = headers_present else {
        return (StatusCode::UNAUTHORIZED, "missing OTA auth headers").into_response();
    };

    // Single-use: removed the instant it's looked up, whether or not it turns out valid,
    // so a captured request/response pair can't be replayed.
    let nonce_was_valid = {
        let mut nonces = state.nonces.lock().await;
        match nonces.remove(&nonce) {
            Some(issued_at) => issued_at.elapsed() < NONCE_TTL,
            None => false,
        }
    };

    if !nonce_was_valid {
        record_auth_failure(&state, ip).await;
        return (StatusCode::UNAUTHORIZED, "unknown or expired nonce").into_response();
    }

    let mut hasher = Sha256::new();
    hasher.update(state.config.ota.password.as_bytes());
    hasher.update(nonce.as_bytes());
    hasher.update(cnonce.as_bytes());
    let expected = hasher.finalize();

    let provided = match hex::decode(&auth) {
        Ok(bytes) => bytes,
        Err(_) => {
            record_auth_failure(&state, ip).await;
            return (StatusCode::UNAUTHORIZED, "malformed auth digest").into_response();
        }
    };

    let digest_matches =
        provided.len() == expected.len() && provided.ct_eq(&expected).unwrap_u8() == 1;

    if !digest_matches {
        record_auth_failure(&state, ip).await;
        return (StatusCode::UNAUTHORIZED, "auth digest mismatch").into_response();
    }

    record_auth_success(&state, ip).await;

    next.run(req).await
}

async fn handle_upload(headers: HeaderMap, body: Bytes) -> Response {
    if body.len() < MIN_BINARY_SIZE {
        return (StatusCode::BAD_REQUEST, "binary too small").into_response();
    }

    let Some(expected_sha256) = headers.get("x-ota-sha256").and_then(|v| v.to_str().ok()) else {
        return (StatusCode::BAD_REQUEST, "missing X-OTA-Sha256 header").into_response();
    };

    let mut hasher = Sha256::new();
    hasher.update(&body);
    let actual_sha256 = hex::encode(hasher.finalize());

    if !actual_sha256.eq_ignore_ascii_case(expected_sha256) {
        warn!("OTA: checksum mismatch, refusing to apply binary");
        return (StatusCode::BAD_REQUEST, "checksum mismatch").into_response();
    }

    if let Some(target) = headers.get("x-ota-target").and_then(|v| v.to_str().ok()) {
        if target != current_platform::CURRENT_PLATFORM {
            warn!(
                "OTA: target mismatch, expected {} got {target}",
                current_platform::CURRENT_PLATFORM
            );
            return (
                StatusCode::BAD_REQUEST,
                format!(
                    "target mismatch: device is {}",
                    current_platform::CURRENT_PLATFORM
                ),
            )
                .into_response();
        }
    }

    info!("OTA: applying verified update ({} bytes)", body.len());

    let bytes = body.to_vec();
    let apply_result = tokio::task::spawn_blocking(move || {
        ubihome_core::features::self_update::apply_new_binary(&bytes)
    })
    .await;

    match apply_result {
        Ok(Ok(())) => {
            info!("OTA: update applied, restarting");
            // Give the response a moment to flush before the process exits. The
            // supervisor (systemd `Restart=always`, or the Windows SCM recovery
            // action configured at install time) brings the process back up on the
            // new binary; returning Err from `run()` here is not an option, since
            // the runtime's task supervisor treats a module error as fatal.
            tokio::spawn(async {
                tokio::time::sleep(Duration::from_millis(300)).await;
                std::process::exit(0);
            });
            (StatusCode::OK, Json(json!({ "status": "accepted" }))).into_response()
        }
        Ok(Err(e)) => {
            error!("OTA: failed to apply update: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to apply update: {e}"),
            )
                .into_response()
        }
        Err(e) => {
            error!("OTA: update task panicked: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error applying update",
            )
                .into_response()
        }
    }
}

#[derive(Clone, Debug)]
pub struct UbiHomePlatform {
    config: CoreConfig,
}

impl Module for UbiHomePlatform {
    fn new(config_string: &str, config_path: &str) -> Result<Self, String> {
        let config =
            ubihome_core::validation::validate_config::<CoreConfig>(config_string, config_path)?;

        Ok(UbiHomePlatform { config })
    }

    fn components(&mut self) -> Vec<UbiComponent> {
        Vec::new()
    }

    fn run(
        &self,
        _sender: Sender<ChangedMessage>,
        _receiver: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let config = self.config.clone();
        info!("Starting OTA receiver on port {}", config.ota.port);

        Box::pin(async move {
            let max_binary_size = config.ota.max_binary_size as usize;
            let app_state = Arc::new(AppState {
                config: config.clone(),
                nonces: Mutex::new(HashMap::new()),
                failed_attempts: Mutex::new(HashMap::new()),
            });

            let upload_route = post(handle_upload)
                .layer(DefaultBodyLimit::max(max_binary_size))
                .route_layer(middleware::from_fn_with_state(
                    app_state.clone(),
                    require_valid_nonce,
                ));

            let app = Router::new()
                .route("/challenge", get(handle_challenge))
                .route("/upload", upload_route)
                .with_state(app_state);

            let addr: SocketAddr = format!("0.0.0.0:{}", config.ota.port).parse().unwrap();
            let socket = TcpSocket::new_v4().unwrap();
            socket.set_reuseaddr(true).unwrap();
            socket.bind(addr).unwrap();
            let listener = socket.listen(128).unwrap();

            info!("OTA listening on http://{addr}");

            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await?;

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ota_config_parsing() {
        let config = r#"
ubihome:
  name: "Test OTA Config"

ota:
  port: 3333
  password: "01234567890123456789012345678901"
"#;

        let ota_module = UbiHomePlatform::new(config, "config.yml");
        assert!(ota_module.is_ok(), "OTA module should parse successfully");

        let module = ota_module.unwrap();
        assert_eq!(module.config.ota.port, 3333, "Port should be 3333");
    }

    #[test]
    fn test_ota_config_defaults() {
        let config = r#"
ubihome:
  name: "Test OTA Config"

ota:
  password: "01234567890123456789012345678901"
"#;

        let ota_module = UbiHomePlatform::new(config, "config.yml");
        assert!(ota_module.is_ok(), "OTA module should parse successfully");

        let module = ota_module.unwrap();
        assert_eq!(module.config.ota.port, 3232, "Port should default to 3232");
        assert_eq!(
            module.config.ota.max_binary_size, 200_000_000,
            "max_binary_size should default to 200_000_000"
        );
    }

    #[test]
    fn test_ota_config_rejects_short_password() {
        let config = r#"
ubihome:
  name: "Test OTA Config"

ota:
  password: "tooshort"
"#;

        let ota_module = UbiHomePlatform::new(config, "config.yml");
        assert!(
            ota_module.is_err(),
            "OTA module should reject a password shorter than 32 characters"
        );
    }
}
