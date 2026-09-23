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

use axum::{extract::{ConnectInfo, DefaultBodyLimit, Request, State}, http::{HeaderValue, header::RETRY_AFTER}, middleware::{self, Next}, response::{IntoResponse, Response}, routing::get, Router, serve::ListenerExt};
use tokio::{net::TcpListener, signal, sync::Semaphore};
use tower::limit::ConcurrencyLimitLayer;
use tower_http::{cors::CorsLayer, csrf::CsrfLayer, limit::RequestBodyLimitLayer, timeout::TimeoutLayer, trace::TraceLayer};
use tracing::info;

use crate::{error::AppError, rate_limit::IpRateLimiter, state::AppState};

const DEFAULT_HOST: Ipv4Addr = Ipv4Addr::LOCALHOST;
const DEFAULT_PORT: u16 = 3000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    init_tracing();

    let addr = SocketAddr::new(IpAddr::V4(parse_ipv4_env("PDFIN_HOST", DEFAULT_HOST)), parse_env("PDFIN_PORT", DEFAULT_PORT));
    let state = AppState {
        pdf_semaphore: Arc::new(Semaphore::new(4)),
        conversion_semaphore: Arc::new(Semaphore::new(2)),
        rate_limiter: IpRateLimiter::from_env(),
    };

    let app = Router::new()
        .route("/health", get(health))
        .merge(routes::api_routes())
        .with_state(state.clone())
        .layer(ConcurrencyLimitLayer::new(8))
        .layer(DefaultBodyLimit::max(500 * 1024 * 1024))
        .layer(RequestBodyLimitLayer::new(500 * 1024 * 1024))
        .layer(TimeoutLayer::with_status_code(axum::http::StatusCode::REQUEST_TIMEOUT, Duration::from_secs(120)))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(CsrfLayer::new())
        .layer(middleware::from_fn(add_security_headers));

    let listener = TcpListener::bind(addr).await?.tap_io(|s| { let _ = s.set_nodelay(true); });
    info!(%addr, "Backend Axum berjalan");

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn health() -> &'static str { "ok" }

async fn add_security_headers(req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    let h = res.headers_mut();
    h.insert("X-Content-Type-Options", HeaderValue::from_static("nosniff"));
    h.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
    h.insert("Cache-Control", HeaderValue::from_static("no-store"));
    res
}

fn parse_env<T: std::str::FromStr + Copy>(name: &str, default: T) -> T { std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default) }
fn parse_ipv4_env(name: &str, default: Ipv4Addr) -> Ipv4Addr { std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default) }

fn init_tracing() { tracing_subscriber::fmt().with_target(false).init(); }

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
}
