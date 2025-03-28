use axum::{
    extract::State,
    routing::{get},
    Router,
    response::{IntoResponse},
    Json,
    http::StatusCode,
};
use std::time::Duration;

use env_logger;
use env_logger::Env;
use clap::Parser;
use os_home::AppState;
use std::sync::Arc;
use log::{error, info};
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

async fn handle_request(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, Json("Hello, World!"))
}

#[derive(Parser)]
#[command(name = "API Mock Server")]
struct Cli {
    #[arg(
        short,
        long,
        default_value = "127.0.0.1:3000",
        help = "Address to bind the server"
    )]
    bind_address: String,

}


#[tokio::main]
async fn main() {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let cli = Cli::parse();

    let bind_address = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| cli.bind_address.clone());
    println!("Listening on http://{}", bind_address);

    let app_state = Arc::new(AppState {
        custom_directory: "test".to_string(),
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




    let listener = TcpListener::bind(bind_address).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await.unwrap();


    info!("Server shutdown complete");
    std::process::exit(0);
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