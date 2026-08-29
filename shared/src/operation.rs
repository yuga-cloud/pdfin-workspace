use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PdfOperation {
    Compress,
    Merge,
    Split,
    Rotate,
    Watermark,
    PageNumbers,

    PdfToWord,
    PdfToExcel,
    PdfToPowerPoint,

    WordToPdf,
    ExcelToPdf,
    PowerPointToPdf,

    JpgToPdf,
    PdfToJpg,
}
