use axum::{Router, routing::post};

use crate::state::AppState;

use crate::features::convert::handlers::{
    excel_to_pdf, jpg_to_pdf, pdf_to_excel, pdf_to_jpg, pdf_to_word, word_to_pdf,
};

/// Endpoint konversi dokumen di API v1.
pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/convert/excel-to-pdf", post(excel_to_pdf::handler))
        .route("/convert/jpg-to-pdf", post(jpg_to_pdf::handler))
        .route("/convert/pdf-to-excel", post(pdf_to_excel::handler))
        .route("/convert/pdf-to-jpg", post(pdf_to_jpg::handler))
        .route("/convert/pdf-to-word", post(pdf_to_word::handler))
        .route("/convert/word-to-pdf", post(word_to_pdf::handler))
}
