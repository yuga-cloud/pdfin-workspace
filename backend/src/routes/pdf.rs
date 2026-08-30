use axum::{Router, routing::post};

use crate::{
    handlers::{
        pdf::{manage_pages, merge_pdfs, rotate},
        pdf_split::split_pdf,
    },
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/rust-api/gabung", post(merge_pdfs))
        .route("/rust-api/pisah", post(split_pdf))
        .route("/rust-api/atur-halaman", post(manage_pages))
        .route("/rust-api/putar", post(rotate))
}
