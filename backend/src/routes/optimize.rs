use axum::{Router, routing::post};

use crate::{
    handlers::optimize::{add_page_numbers, add_watermark, compress_pdf},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/rust-api/kompres", post(compress_pdf))
        .route("/rust-api/watermark", post(add_watermark))
        .route("/rust-api/page-numbers", post(add_page_numbers))
}
