mod engines;
mod error;
mod features;
mod handlers;
mod routes;
mod state;

use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use axum::{Router, extract::DefaultBodyLimit, routing::get, serve::ListenerExt};
use tokio::{net::TcpListener, sync::Semaphore};
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::info;

use crate::state::AppState;

const SERVER_HOST: Ipv4Addr = Ipv4Addr::LOCALHOST;
const SERVER_PORT: u16 = 3000;

const MAX_REQUEST_BODY_SIZE: usize = 50 * 1024 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    init_tracing();

    let server_addr = SocketAddr::new(IpAddr::V4(SERVER_HOST), SERVER_PORT);

    let pdf_concurrency = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);

    let state = AppState {
        pdf_semaphore: Arc::new(Semaphore::new(pdf_concurrency)),
    };

    info!(
        address = %server_addr,
        pdf_concurrency,
        max_request_mb = MAX_REQUEST_BODY_SIZE / (1024 * 1024),
        request_timeout_seconds = REQUEST_TIMEOUT.as_secs(),
        "Menyiapkan backend"
    );

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .merge(routes::api_routes())
        .with_state(state)
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BODY_SIZE))
        .layer(RequestBodyLimitLayer::new(MAX_REQUEST_BODY_SIZE))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            REQUEST_TIMEOUT,
        ))
        .layer(
            TraceLayer::new_for_http()
                .on_request(|request: &axum::http::Request<_>, _span: &tracing::Span| {
                    tracing::info!(
                        method = %request.method(),
                        uri = %request.uri(),
                        "HTTP request masuk"
                    );
                })
                .on_response(
                    |response: &axum::http::Response<_>,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::info!(
                            status = %response.status(),
                            latency_ms = latency.as_millis(),
                            "HTTP response selesai"
                        );
                    },
                ),
        )
        .layer(cors);

    let listener = TcpListener::bind(server_addr).await?.tap_io(|stream| {
        if let Err(error) = stream.set_nodelay(true) {
            tracing::debug!(
                %error,
                "Gagal mengaktifkan TCP_NODELAY"
            );
        }
    });

    info!(
        address = %server_addr,
        "Backend Axum berjalan"
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Backend Axum berhenti");

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .compact()
        .init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("gagal memasang Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("gagal memasang SIGTERM handler");

        signal.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Menerima SIGINT/Ctrl+C");
        }

        _ = terminate => {
            info!("Menerima SIGTERM");
        }
    }
}
