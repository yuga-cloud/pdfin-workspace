mod engines;
mod error;
mod features;
mod handlers;
mod openapi;
mod rate_limit;
mod routes;
mod state;
mod zip;

use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use axum::{
    Router,
    extract::DefaultBodyLimit,
    extract::{ConnectInfo, Request, State},
    http::{HeaderValue, header::RETRY_AFTER},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    serve::ListenerExt,
};
use tokio::{net::TcpListener, sync::Semaphore};
use tower::limit::ConcurrencyLimitLayer;

use tower_http::{
    cors::CorsLayer, csrf::CsrfLayer, limit::RequestBodyLimitLayer, timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::info;

use crate::{error::AppError, rate_limit::IpRateLimiter, state::AppState};

const DEFAULT_HOST: Ipv4Addr = Ipv4Addr::LOCALHOST;
const DEFAULT_PORT: u16 = 3000;
const DEFAULT_MAX_REQUEST_BODY_SIZE_MB: usize = 500;
const DEFAULT_REQUEST_TIMEOUT_SECONDS: u64 = 120;
const DEFAULT_MAX_CONCURRENCY: usize = 4;
const DEFAULT_MAX_CONVERSION_CONCURRENCY: usize = 2;
const DEFAULT_MAX_IN_FLIGHT_REQUESTS_PER_PDF_WORKER: usize = 2;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    init_tracing();

    let host = parse_ipv4_env("PDFIN_HOST", DEFAULT_HOST);
    let port = parse_env("PDFIN_PORT", DEFAULT_PORT);
    let server_addr = SocketAddr::new(IpAddr::V4(host), port);

    let state = AppState {
        pdf_semaphore: Arc::new(Semaphore::new(DEFAULT_MAX_CONCURRENCY)),
        conversion_semaphore: Arc::new(Semaphore::new(DEFAULT_MAX_CONVERSION_CONCURRENCY)),
        rate_limiter: IpRateLimiter::from_env(),
    };

    let app = Router::new()
        .route("/health", get(health))
        .merge(routes::api_routes())
        .with_state(state);

    let listener = TcpListener::bind(server_addr).await?.tap_io(|stream| {
        let _ = stream.set_nodelay(true);
    });

    info!(address = %server_addr, "Backend Axum berjalan");

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}
