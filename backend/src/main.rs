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
const DEFAULT_MAX_CONVERSION_CONCURRENCY: usize = 2;
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
    let configured_conversion_concurrency =
        parse_env("PDFIN_MAX_CONVERSION_CONCURRENCY", DEFAULT_MAX_CONVERSION_CONCURRENCY);
    let conversion_concurrency = cpu_count.min(configured_conversion_concurrency.max(1));
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
        conversion_semaphore: Arc::new(Semaphore::new(conversion_concurrency)),
    };

    info!(
        address = %server_addr,
        cpu_count,
        pdf_concurrency,
        conversion_concurrency,
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

    axum::serve(listener, app).await?;
    Ok(())
}
