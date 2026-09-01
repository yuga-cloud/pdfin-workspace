use std::sync::Arc;

use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct AppState {
    pub pdf_semaphore: Arc<Semaphore>,
    pub conversion_semaphore: Arc<Semaphore>,
}
