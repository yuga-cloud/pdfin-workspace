mod engines;
mod error;
mod features;
mod handlers;
mod openapi;
mod rate_limit;
mod routes;
mod state;
mod zip;

use std::{error::Error, net::{IpAddr, Ipv4Addr, SocketAddr}, sync::Arc, time::Duration};

use axum::{
    extract::{ConnectInfo, DefaultBodyLimit, Request, State},
    http::{header::RETRY_AFTER, HeaderValue},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    serve::ListenerExt,
    Router,
};
use tokio::{net::TcpListener, signal, sync::Semaphore};
use tower::limit::ConcurrencyLimitLayer;
use tower_http::{
    cors::CorsLayer,
    csrf::CsrfLayer,
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::info;

use crate::{error::AppError, rate_limit::IpRateLimiter, state::AppState};

const DEFAULT_HOST: Ipv4Addr = Ipv4Addr::LOCALHOST;
const DEFAULT_PORT: u16 = 3000;
const DEFAULT_BODY_LIMIT_MB: usize = 500;
const DEFAULT_TIMEOUT_SECONDS: u64 = 120;
const DEFAULT_MAX_CONCURRENCY: usize = 8;
const DEFAULT_PDF_CONCURRENCY: usize = 4;
const DEFAULT_CONVERSION_CONCURRENCY: usize = 2;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    init_tracing();

    let addr = SocketAddr::new(
        IpAddr::V4(parse_ipv4_env("PDFIN_HOST", DEFAULT_HOST)),
        parse_env("PDFIN_PORT", DEFAULT_PORT),
    );

    let body_limit = parse_env("PDFIN_MAX_REQUEST_MB", DEFAULT_BODY_LIMIT_MB)
        .max(1)
        .saturating_mul(1024 * 1024);

    let state = AppState {
        pdf_semaphore: Arc::new(Semaphore::new(DEFAULT_PDF_CONCURRENCY)),
        conversion_semaphore: Arc::new(Semaphore::new(DEFAULT_CONVERSION_CONCURRENCY)),
        rate_limiter: IpRateLimiter::from_env(),
    };

    let app = Router::new()
        .route("/health", get(health))
        .merge(routes::api_routes())
        .with_state(state.clone())
        .layer(ConcurrencyLimitLayer::new(DEFAULT_MAX_CONCURRENCY))
        .layer(DefaultBodyLimit::max(body_limit))
        .layer(RequestBodyLimitLayer::new(body_limit))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(DEFAULT_TIMEOUT_SECONDS),
        ))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::very_permissive())
        .layer(CsrfLayer::new())
        .layer(middleware::from_fn(add_security_headers))
        .layer(middleware::from_fn_with_state(
            state,
            enforce_rate_limit,
        ));

    let listener = TcpListener::bind(addr).await?.tap_io(|stream| {
        let _ = stream.set_nodelay(true);
    });

    info!(address = %addr, "Backend Axum berjalan");

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

async fn add_security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert("X-Content-Type-Options", HeaderValue::from_static("nosniff"));
    headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
    headers.insert("Cache-Control", HeaderValue::from_static("no-store"));

    response
}

async fn enforce_rate_limit(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let client_ip = state.rate_limiter.client_ip(addr.ip(), request.headers());

    if !state.rate_limiter.allow(client_ip) {
        let mut response = AppError::too_many_requests(
            "rate_limited",
            "Terlalu banyak request.",
        )
        .into_response();

        response
            .headers_mut()
            .insert(RETRY_AFTER, HeaderValue::from_static("1"));

        return response;
    }

    next.run(request).await
}

fn parse_env<T: std::str::FromStr + Copy>(name: &str, default: T) -> T {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn parse_ipv4_env(name: &str, default: Ipv4Addr) -> Ipv4Addr {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn init_tracing() {
    tracing_subscriber::fmt().with_target(false).init();
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
}
