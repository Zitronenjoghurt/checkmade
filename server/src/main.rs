use crate::config::Config;
use crate::layers::metrics::metrics_middleware;
use axum::routing::get;
use axum::{http, middleware, Router};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod api;
mod config;
mod error;
mod integrations;
mod layers;
mod state;
mod websocket;

#[tokio::main]
async fn main() {
    init_logging();
    info!("Starting server...");

    let config = Config::from_env().unwrap();
    let static_dir = config.static_dir.clone();

    let state = state::ServerState::new(config).await.unwrap();
    let api = api::build();

    let session_layer = layers::session::build_session_layer(&state).await;

    let prometheus_handle = PrometheusBuilder::new().install_recorder().unwrap();
    let collector = metrics_process::Collector::default();
    collector.describe();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(15));
        loop {
            interval.tick().await;
            collector.collect();
        }
    });

    let router = Router::new()
        .route("/ws", get(websocket::handler::ws_handler))
        .nest("/api", api)
        .fallback_service(ServeDir::new(static_dir))
        .layer(session_layer)
        .layer(middleware::from_fn(metrics_middleware))
        .layer(SetResponseHeaderLayer::overriding(
            http::header::HeaderName::from_static("cross-origin-opener-policy"),
            http::header::HeaderValue::from_static("same-origin"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            http::header::HeaderName::from_static("cross-origin-embedder-policy"),
            http::header::HeaderValue::from_static("require-corp"),
        ))
        .with_state(state);
    let metrics_router = Router::new().route(
        "/metrics",
        get(move || std::future::ready(prometheus_handle.render())),
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], 51292));
    let metrics_addr = SocketAddr::from(([0, 0, 0, 0], 9092));

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("Listening on {}", addr);

    let metrics_listener = tokio::net::TcpListener::bind(&metrics_addr).await.unwrap();
    info!("Metrics on {}", metrics_addr);

    tokio::select! {
        r = axum::serve(listener, router).with_graceful_shutdown(shutdown_signal()) => r.unwrap(),
        r = axum::serve(metrics_listener, metrics_router) => r.unwrap(),
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutting down...");
}

fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}

pub fn server_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
