mod detector;
mod extractor;
pub mod model;

pub use detector::detect_tables;
pub use extractor::extract_pdf_words;
pub use model::Table;
