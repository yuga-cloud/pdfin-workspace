use axum::Router;

use crate::state::AppState;

pub mod optimize;
pub mod pdf;

pub fn api_routes() -> Router<AppState> {
    Router::<AppState>::new()
        .merge(pdf::routes())
        .merge(optimize::routes())
        .merge(crate::features::convert::routes::routes())
}
