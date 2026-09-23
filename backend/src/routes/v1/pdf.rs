use axum::{Router, routing::post};

use crate::{
    handlers::{
        compress::compress_pdf,
        optimize::{add_page_numbers, add_watermark},
        pages::manage_pages,
        pdf::{merge_pdfs, rotate},
        pdf_split::split_pdf,
    },
    state::AppState,
};

/// Operasi PDF di API v1: merge, split, atur halaman, putar, kompres,
/// watermark, dan nomor halaman. Semua endpoint multipart sinkron dan
/// mengembalikan file hasil langsung.
pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/pdf/merge", post(merge_pdfs))
        .route("/pdf/split", post(split_pdf))
        .route("/pdf/pages", post(manage_pages))
        .route("/pdf/rotate", post(rotate))
        .route("/pdf/compress", post(compress_pdf))
        .route("/pdf/watermark", post(add_watermark))
        .route("/pdf/page-numbers", post(add_page_numbers))
}
