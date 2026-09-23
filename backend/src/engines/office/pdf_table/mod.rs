mod detector;
mod extractor;
pub mod model;

pub use detector::detect_tables;
pub use extractor::{extract_pdf_words, run_pdfium_worker};
pub use model::Table;
