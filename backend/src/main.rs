mod engines;
mod error;
mod features;
mod handlers;
mod routes;
mod state;
mod zip;

use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use axum::{Router, extract::DefaultBodyLimit, routing::get, serve::ListenerExt};
use tokio::{net::TcpListener, sync::Semaphore};
use tower::limit::ConcurrencyLimitLayer;
use tower_http::{
    cors::CorsLayer, limit::RequestBodyLimitLayer, timeout::TimeoutLayer, trace::TraceLayer,
};
use tracing::info;

use crate::state::AppState;

const DEFAULT_HOST: Ipv4Addr = Ipv4Addr::LOCALHOST;
const DEFAULT_PORT: u16 = 3000;
const DEFAULT_MAX_REQUEST_BODY_SIZE_MB: usize = 50;
const DEFAULT_REQUEST_TIMEOUT_SECONDS: u64 = 120;
const DEFAULT_MAX_CONCURRENCY: usize = 4;
const DEFAULT_MAX_IN_FLIGHT_REQUESTS_PER_PDF_WORKER: usize = 2;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    init_tracing();

    let host = parse_ipv4_env("PDFIN_HOST", DEFAULT_HOST);
    let port = parse_env("PDFIN_PORT", DEFAULT_PORT);
    let max_request_body_size_mb =
        parse_env("PDFIN_MAX_REQUEST_MB", DEFAULT_MAX_REQUEST_BODY_SIZE_MB).max(1);
    let max_request_body_size = max_request_body_size_mb.saturating_mul(1024 * 1024);
    let request_timeout = Duration::from_secs(
        parse_env(
            "PDFIN_REQUEST_TIMEOUT_SECONDS",
            DEFAULT_REQUEST_TIMEOUT_SECONDS,
        )
        .max(1),
    );

    let cpu_count = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    let configured_max_concurrency = parse_env("PDFIN_MAX_CONCURRENCY", DEFAULT_MAX_CONCURRENCY);
    let pdf_concurrency = cpu_count.min(configured_max_concurrency.max(1));
    let default_max_in_flight_requests = pdf_concurrency
        .saturating_mul(DEFAULT_MAX_IN_FLIGHT_REQUESTS_PER_PDF_WORKER)
        .max(1);
    let max_in_flight_requests = parse_env(
        "PDFIN_MAX_IN_FLIGHT_REQUESTS",
        default_max_in_flight_requests,
    )
    .max(1);

    let server_addr = SocketAddr::new(IpAddr::V4(host), port);

    let state = AppState {
        pdf_semaphore: Arc::new(Semaphore::new(pdf_concurrency)),
    };

    info!(
        address = %server_addr,
        cpu_count,
        pdf_concurrency,
        max_in_flight_requests,
        max_request_mb = max_request_body_size_mb,
        request_timeout_seconds = request_timeout.as_secs(),
        "Menyiapkan backend"
    );

    let cors = build_cors_layer()?;

    let app = Router::new()
        .route("/health", get(health))
        .merge(routes::api_routes())
        .with_state(state)
        .layer(ConcurrencyLimitLayer::new(max_in_flight_requests))
        .layer(DefaultBodyLimit::max(max_request_body_size))
        .layer(RequestBodyLimitLayer::new(max_request_body_size))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            request_timeout,
        ))
        .layer(
            TraceLayer::new_for_http()
                .on_request(|request: &axum::http::Request<_>, _span: &tracing::Span| {
                    tracing::info!(
                        method = %request.method(),
                        uri = %request.uri().path(),
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
            tracing::debug!(%error, "Gagal mengaktifkan TCP_NODELAY");
        }
    });

    info!(address = %server_addr, "Backend Axum berjalan");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Backend Axum berhenti");

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

fn parse_env<T>(name: &str, default: T) -> T
where
    T: std::str::FromStr + Copy,
{
    match std::env::var(name) {
        Ok(value) => value.parse::<T>().unwrap_or_else(|_| {
            tracing::warn!(
                variable = name,
                "Nilai environment tidak valid; memakai default"
            );
            default
        }),
        Err(_) => default,
    }
}

fn parse_ipv4_env(name: &str, default: Ipv4Addr) -> Ipv4Addr {
    match std::env::var(name) {
        Ok(value) => value.parse::<Ipv4Addr>().unwrap_or_else(|_| {
            tracing::warn!(variable = name, "Alamat IPv4 tidak valid; memakai default");
            default
        }),
        Err(_) => default,
    }
}

fn build_cors_layer() -> Result<CorsLayer, Box<dyn Error + Send + Sync>> {
    let Some(origin) = std::env::var("PDFIN_CORS_ORIGIN")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
    else {
        return Ok(CorsLayer::new()
            .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
            .allow_headers([axum::http::header::CONTENT_TYPE]));
    };

    let origin = origin.parse::<axum::http::HeaderValue>()?;

    Ok(CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers([axum::http::header::CONTENT_TYPE]))
}

fn init_tracing() {
    let env_filter = std::env::var("RUST_LOG")
        .ok()
        .and_then(|value| value.parse::<tracing::Level>().ok());

    let builder = tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .compact();

    if let Some(level) = env_filter {
        builder.with_max_level(level).init();
    } else {
        builder.with_max_level(tracing::Level::INFO).init();
    }
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
