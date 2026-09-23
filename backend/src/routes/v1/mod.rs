use axum::Router;

use crate::state::AppState;

pub mod conversions;
pub mod files;
pub mod health;
pub mod jobs;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(files::routes())
        .merge(conversions::routes())
        .merge(jobs::routes())
        .merge(health::routes())
}
