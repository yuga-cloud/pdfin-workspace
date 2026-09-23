use axum::Router;

use crate::state::AppState;

pub mod docs;
pub mod optimize;
pub mod pdf;
pub mod v1;

pub fn api_routes() -> Router<AppState> {
    Router::<AppState>::new()
        .merge(docs::routes())
        .merge(pdf::routes())
        .merge(optimize::routes())
        .merge(crate::features::convert::routes::routes())
        .nest("/api/v1", v1::routes())
}
