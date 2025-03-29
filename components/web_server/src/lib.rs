use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use std::time::Duration;

use log::{info, warn};
use std::sync::Arc;
use tokio::signal;
use tokio::{net::TcpListener};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

#[derive(Clone, Debug)]
struct AppState {
    pub custom_directory: String,
}


async fn handle_request(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, Json("Hello, World!"))
}

pub async fn start(){
    let bind_address = "0.0.0.0:8080";

    let app_state = Arc::new(AppState {
        custom_directory: "bla".to_string()
    });

    let app = Router::new()
    .route("/", get(handle_request))
    .layer((
        TraceLayer::new_for_http(),
        // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
        // requests don't hang forever.
        TimeoutLayer::new(Duration::from_secs(10)),
    ))
    .with_state(app_state.clone());

    let listener = TcpListener::bind(bind_address.clone()).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    info!("Listening on http://{}", bind_address);
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
