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
use os_home::Config;
use std::sync::Arc;
use log::{info};
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use rumqttc::{AsyncClient, MqttOptions, QoS, Event};
use std::fs;

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

    #[arg(
        short,
        long,
        help = "Configuration file if the default configuration should not be used."
    )]
    configuration_file: String,
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let cli = Cli::parse();

    let bind_address = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| cli.bind_address.clone());
    println!("Listening on http://{}", bind_address);

    // Read the YAML configuration file
    let config: Config = read_config("config.yaml").expect("Failed to read config.yaml");

    let app_state = Arc::new(AppState {
        custom_directory: "bla".to_string(),
        config,
    });

    // Start MQTT client in a separate task
    tokio::spawn(async {
        start_mqtt_client().await;
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

// Function to read the YAML configuration file
fn read_config(path: &str) -> Result<Config, serde_yaml::Error> {
    let content = fs::read_to_string(path).expect("Failed to read the file");
    serde_yaml::from_str(&content)
}

async fn start_mqtt_client() {
    let mut mqttoptions = MqttOptions::new("test-client", "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // Subscribe to a test topic
    client.subscribe("test/topic", QoS::AtMostOnce).await.unwrap();

    // Publish a test message
    client.publish("test/topic", QoS::AtMostOnce, false, "Hello MQTT!").await.unwrap();

    // Handle incoming messages
    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(incoming)) => {
                    println!("Incoming: {:?}", incoming);
                }
                Ok(Event::Outgoing(_)) => {}
                Err(e) => {
                    eprintln!("Error in MQTT event loop: {:?}", e);
                    break;
                }
            }
        }
    });
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