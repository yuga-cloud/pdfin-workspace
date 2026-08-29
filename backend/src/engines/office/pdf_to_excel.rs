use std::{fs, path::PathBuf};

use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook};

use crate::engines::common::validate_input;
use crate::engines::office::ocr::{renderer, tesseract};

use super::pdf_table::{Table, detect_tables, extract_pdf_words, model::PdfWord};

const MIN_COLUMN_WIDTH: f64 = 10.0;
const MAX_COLUMN_WIDTH: f64 = 60.0;

const HEADER_COLOR: u32 = 0x44C7E6;

/* -------------------------------------------------------------------------- */
/* PUBLIC API                                                                 */
/* -------------------------------------------------------------------------- */

pub fn pdf_to_excel(pdf_bytes: &[u8]) -> Result<Vec<u8>, String> {
    validate_input(pdf_bytes, "PDF")?;

    tracing::info!("Memulai PDF → Excel");

    /*
     * Tahap 1:
     * PDFium mencoba membaca text layer PDF.
     */
    let tables = match extract_pdf_words(pdf_bytes) {
        Ok(pages) => {
            tracing::info!(
                pages = pages.len(),
                "PDF text berhasil diekstrak menggunakan PDFium"
            );

            /*
             * Tahap 2:
             * Deteksi struktur tabel berdasarkan posisi text.
             */
            detect_tables(&pages)
        }
        Err(error) => {
            tracing::warn!(
                %error,
                "Ekstraksi text PDFium gagal, langsung coba OCR fallback"
            );

            Vec::new()
        }
    };

    if !tables.is_empty() {
        tracing::info!(
            tables = tables.len(),
            "Struktur tabel berhasil ditemukan menggunakan PDFium"
        );

        return build_workbook(&tables);
    }

    /*
     * Jika PDFium tidak menemukan struktur tabel,
     * gunakan OCR sebagai fallback.
     */
    tracing::warn!("Struktur tabel tidak ditemukan menggunakan PDFium, menggunakan OCR fallback");

    pdf_to_excel_ocr(pdf_bytes)
}

/* -------------------------------------------------------------------------- */
/* OCR FALLBACK                                                               */
/* -------------------------------------------------------------------------- */

/// Mengonversi item OCR (OcrTextItem) menjadi PdfWord agar dapat
/// diproses oleh algoritma detektor tabel yang sama dengan PDF native.
fn ocr_items_to_pdf_words(items: &[tesseract::OcrTextItem]) -> Vec<PdfWord> {
    items
        .iter()
        .map(|item| {
            PdfWord::new(
                item.text.clone(),
                item.x as f32,
                (item.x + item.width) as f32,
                item.y as f32,
                (item.y + item.height) as f32,
            )
        })
        .collect()
}

fn pdf_to_excel_ocr(pdf_bytes: &[u8]) -> Result<Vec<u8>, String> {
    tracing::info!("Memulai OCR fallback PDF → Excel");

    let image_paths = renderer::render_pdf_pages(pdf_bytes)?;

    if image_paths.is_empty() {
        return Err("OCR renderer tidak menghasilkan halaman gambar.".to_owned());
    }

    let mut pages_of_words = Vec::with_capacity(image_paths.len());

    for (index, image_path) in image_paths.iter().enumerate() {
        tracing::debug!(
            page = index + 1,
            image = %image_path.display(),
            "Menjalankan Tesseract OCR"
        );

        let items = match tesseract::ocr_image(image_path) {
            Ok(items) => items,
            Err(error) => {
                tracing::warn!(
                    page = index + 1,
                    %error,
                    "Tesseract gagal memproses halaman ini, dilewati"
                );

                pages_of_words.push(Vec::new());
                continue;
            }
        };

        if items.is_empty() {
            tracing::warn!(page = index + 1, "Tesseract tidak menemukan text");

            pages_of_words.push(Vec::new());
            continue;
        }

        /*
         * Ubah OcrTextItem menjadi PdfWord agar seragam
         * dengan pipeline deteksi tabel native.
         */
        let words = ocr_items_to_pdf_words(&items);

        tracing::debug!(
            page = index + 1,
            words = words.len(),
            "OCR page berhasil diproses ke PdfWord"
        );

        pages_of_words.push(words);
    }

    /*
     * Cleanup dilakukan setelah seluruh OCR selesai
     * supaya file gambar masih tersedia selama proses pembacaan.
     */
    cleanup_ocr_files(&image_paths);

    /*
     * Jalankan detektor tabel yang sama persis dengan PDF native.
     */
    let tables = detect_tables(&pages_of_words);

    if tables.is_empty() {
        return Err("OCR tidak menghasilkan struktur tabel yang valid.".to_owned());
    }

    tracing::info!(
        tables = tables.len(),
        "OCR berhasil mendeteksi struktur tabel"
    );

    build_workbook(&tables)
}

/* -------------------------------------------------------------------------- */
/* OCR CLEANUP                                                               */
/* -------------------------------------------------------------------------- */

fn cleanup_ocr_files(image_paths: &[PathBuf]) {
    let Some(parent) = image_paths.first().and_then(|path| path.parent()) else {
        return;
    };

    if let Err(error) = fs::remove_dir_all(parent) {
        tracing::warn!(
            path = %parent.display(),
            %error,
            "Gagal membersihkan direktori OCR sementara"
        );
    } else {
        tracing::debug!(
            path = %parent.display(),
            "Direktori OCR sementara berhasil dibersihkan"
        );
    }
}

/* -------------------------------------------------------------------------- */
/* TABLE → XLSX WORKBOOK BUILDER                                              */
/* -------------------------------------------------------------------------- */

fn build_workbook(tables: &[Table]) -> Result<Vec<u8>, String> {
    let mut workbook = Workbook::new();

    for (table_index, table) in tables.iter().enumerate() {
        let rows = &table.rows;

        if rows.is_empty() {
            continue;
        }

        let worksheet = workbook.add_worksheet();

        let sheet_name = format!("Table {}", table_index + 1);

        worksheet
            .set_name(&sheet_name)
            .map_err(|error| format!("Gagal memberi nama worksheet: {error}"))?;

        write_rows_to_sheet(worksheet, rows)?;
    }

    workbook
        .save_to_buffer()
        .map_err(|error| format!("Gagal membuat XLSX: {error}"))
}

/* -------------------------------------------------------------------------- */
/* XLSX WRITER                                                                */
/* -------------------------------------------------------------------------- */

fn write_rows_to_sheet(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    rows: &[Vec<String>],
) -> Result<(), String> {
    let column_count = rows.iter().map(Vec::len).max().unwrap_or(0);

    if column_count == 0 {
        return Ok(());
    }

    if column_count > u16::MAX as usize {
        return Err("Jumlah kolom PDF terlalu besar untuk Excel.".to_owned());
    }

    if rows.len() > u32::MAX as usize {
        return Err("Jumlah baris PDF terlalu besar untuk Excel.".to_owned());
    }

    let header_format = Format::new()
        .set_bold()
        .set_text_wrap()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin)
        .set_background_color(rust_xlsxwriter::Color::RGB(HEADER_COLOR));

    let text_format = Format::new()
        .set_text_wrap()
        .set_align(FormatAlign::Left)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);

    let number_format = Format::new()
        .set_align(FormatAlign::Right)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);

    let mut widths = vec![MIN_COLUMN_WIDTH; column_count];

    /*
     * TAHAP BARU: Deteksi Header Asli Dinamis
     * Mencari baris mana yang merupakan header utama (seperti baris "No" / "Date In").
     * Cirinya: Kolom index 0 terisi, dan baris ini punya minimal 2 kolom yg terisi.
     */
    let mut actual_header_row = 0;
    for (i, row) in rows.iter().enumerate() {
        let starts_on_left = row.first().is_some_and(|cell| !cell.trim().is_empty());
        let filled_count = row.iter().filter(|c| !c.trim().is_empty()).count();

        if starts_on_left && filled_count >= 2 {
            actual_header_row = i;
            break;
        }
    }

    for (row_index, row) in rows.iter().enumerate() {
        for (column_index, value) in row.iter().enumerate() {
            let width = value.chars().count() as f64 + 2.0;

            widths[column_index] = widths[column_index].max(width).min(MAX_COLUMN_WIDTH);

            let row_number = row_index as u32;
            let column_number = column_index as u16;

            /*
             * Baris dinamis dianggap sebagai header untuk pewarnaan biru
             */
            if row_index == actual_header_row {
                worksheet
                    .write_string_with_format(row_number, column_number, value, &header_format)
                    .map_err(|error| format!("Gagal menulis header Excel: {error}"))?;
            } else if let Some(number) = parse_number(value) {
                worksheet
                    .write_number_with_format(row_number, column_number, number, &number_format)
                    .map_err(|error| format!("Gagal menulis angka Excel: {error}"))?;
            } else {
                worksheet
                    .write_string_with_format(row_number, column_number, value, &text_format)
                    .map_err(|error| format!("Gagal menulis cell Excel: {error}"))?;
            }
        }
    }

    /*
     * Atur lebar kolom secara otomatis berdasarkan panjang text.
     */
    for (column_index, width) in widths.iter().enumerate() {
        worksheet
            .set_column_width(
                column_index as u16,
                (*width).clamp(MIN_COLUMN_WIDTH, MAX_COLUMN_WIDTH),
            )
            .map_err(|error| format!("Gagal mengatur lebar kolom: {error}"))?;
    }

    /*
     * Freeze baris tepat di bawah header asli
     */
    worksheet
        .set_freeze_panes(actual_header_row as u32 + 1, 0)
        .map_err(|error| format!("Gagal mengatur freeze panes: {error}"))?;

    Ok(())
}

/* -------------------------------------------------------------------------- */
/* NUMBER PARSER                                                              */
/* -------------------------------------------------------------------------- */

fn parse_number(value: &str) -> Option<f64> {
    let value = value.trim();

    if value.is_empty() {
        return None;
    }

    value.parse::<f64>().ok()
}
