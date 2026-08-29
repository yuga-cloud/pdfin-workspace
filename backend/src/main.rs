use std::env;
use axum::{
    routing::{get, post},
    Router,
    extract::DefaultBodyLimit,
    serve::ListenerExt,
};
use tokio::net::TcpListener;
use tower_http::{
    cors::{CorsLayer, Any, AllowOrigin},
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::info;
use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Duration,
    str::FromStr,
};
use tokio::sync::Semaphore;

mod engines;
mod error;
mod features;
mod handlers;
mod routes;
mod state;

use crate::state::AppState;

// Configuration with environment variable support
struct Config {
    server_host: IpAddr,
    server_port: u16,
    max_request_body_size: usize,
    request_timeout: Duration,
    cors_origin: AllowOrigin,
    rust_log: String,
}

impl Config {
    fn from_env() -> Self {
        let server_host = env::var("SERVER_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string())
            .parse::<IpAddr>()
            .unwrap_or_else(|_| "127.0.0.1".parse().unwrap());
        
        let server_port = env::var("SERVER_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(3000);
        
        let max_request_body_size = env::var("MAX_REQUEST_BODY_MB")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(50)
            * 1024
            * 1024;
        
        let request_timeout_secs = env::var("REQUEST_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(120);
        
        let cors_origin = env::var("CORS_ORIGIN")
            .ok()
            .and_then(|origin| {
                origin.parse::<AllowOrigin>()
                    .ok()
                    .or_else(|| Some(AllowOrigin::any()))
            })
            .unwrap_or_else(|| AllowOrigin::any());
        
        let rust_log = env::var("RUST_LOG")
            .unwrap_or_else(|_| "info".to_string());
        
        Self {
            server_host,
            server_port,
            max_request_body_size,
            request_timeout: Duration::from_secs(request_timeout_secs),
            cors_origin,
            rust_log,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let config = Config::from_env();
    init_tracing(&config.rust_log);
    
    let server_addr = SocketAddr::new(config.server_host, config.server_port);
    
    let pdf_concurrency = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    
    let state = AppState {
        pdf_semaphore: Arc::new(Semaphore::new(pdf_concurrency)),
    };
    
    info!(
        address = %server_addr,
        pdf_concurrency,
        max_request_mb = config.max_request_body_size / (1024 * 1024),
        request_timeout_seconds = config.request_timeout.as_secs(),
        "Server setup starting"
    );
    
    let cors = CorsLayer::new()
        .allow_origin(config.cors_origin)
        .allow_methods(Any)
        .allow_headers(Any);
    
    let app = Router::new()
        .route("/health", get(health))
        .merge(routes::api_routes())
        .with_state(state)
        .layer(DefaultBodyLimit::max(config.max_request_body_size))
        .layer(RequestBodyLimitLayer::new(config.max_request_body_size))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            config.request_timeout,
        ))
        .layer(
            TraceLayer::new_for_http()
                .on_request(|request: &axum::http::Request<_>, _span: &tracing::Span| {
                    tracing::info!(
                        method = %request.method(),
                        uri = %request.uri(),
                        "HTTP request received"
                    );
                })
                .on_response(
                    |response: &axum::http::Response<_>,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::info!(
                            status = %response.status(),
                            latency_ms = latency.as_millis(),
                            "HTTP response completed"
                        );
                    },
                ),
        )
        .layer(cors);
    
    let listener = TcpListener::bind(server_addr).await?.tap_io(|stream| {
        if let Err(error) = stream.set_nodelay(true) {
            tracing::debug!(
                %error,
                "Failed to enable TCP_NODELAY"
            );
        }
    });
    
    info!(
        address = %server_addr,
        "Axum server started"
    );
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    info!("Server shutdown complete");
    
    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

fn init_tracing(rust_log: &str) {
    if let Ok(level) = rust_log.parse() {
        tracing_subscriber::fmt()
            .with_max_level(level)
            .with_target(false)
            .with_thread_ids(true)
            .with_thread_names(true)
            .compact()
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_target(false)
            .with_thread_ids(true)
            .with_thread_names(true)
            .compact()
            .init();
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };
    
    #[cfg(unix)]
    let terminate = async {
        let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler");
        
        signal.recv().await;
    };
    
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    
    tokio::select! {
        _ = ctrl_c => {
            info!("Received SIGINT/Ctrl+C");
        }
        
        _ = terminate => {
            info!("Received SIGTERM");
        }
    }
}
