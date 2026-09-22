use axum::{Router, routing::post};

use crate::state::AppState;

use super::handlers::{
    excel_to_pdf, jpg_to_pdf, pdf_to_excel, pdf_to_jpg, pdf_to_word, word_to_pdf,
};

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/rust-api/excel-to-pdf", post(excel_to_pdf::handler))
        .route("/rust-api/jpg-to-pdf", post(jpg_to_pdf::handler))
        .route("/rust-api/pdf-to-excel", post(pdf_to_excel::handler))
        .route("/rust-api/pdf-to-jpg", post(pdf_to_jpg::handler))
        .route("/rust-api/pdf-to-word", post(pdf_to_word::handler))
        .route("/rust-api/word-to-pdf", post(word_to_pdf::handler))
}
