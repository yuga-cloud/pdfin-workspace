use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::rate_limit::IpRateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub pdf_semaphore: Arc<Semaphore>,
    pub conversion_semaphore: Arc<Semaphore>,
    pub rate_limiter: IpRateLimiter,
}
