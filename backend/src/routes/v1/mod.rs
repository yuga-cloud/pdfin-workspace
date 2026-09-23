use axum::Router;

use crate::state::AppState;

pub mod convert;
pub mod meta;
pub mod pdf;

/// Router untuk API publik versi 1, di-mount oleh `routes::api_routes()`
/// di bawah prefix `/api/v1`.
pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .merge(pdf::routes())
        .merge(convert::routes())
        .merge(meta::routes())
}
