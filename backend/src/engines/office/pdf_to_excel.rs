use std::{fs, path::PathBuf};

use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook};

use crate::engines::common::validate_input;
use crate::engines::office::ocr::{renderer, tesseract};

use super::pdf_table::{Table, detect_tables, extract_pdf_words, model::PdfWord};

const MIN_COLUMN_WIDTH: f64 = 10.0;
const MAX_COLUMN_WIDTH: f64 = 60.0;
const MAX_OCR_TABLES: usize = 100;
const MAX_XLSX_OUTPUT_BYTES: usize = 128 * 1024 * 1024;
const MAX_WORKBOOK_CELLS: usize = 2_000_000;

const HEADER_COLOR: u32 = 0x44C7E6;

pub fn pdf_to_excel(pdf_bytes: &[u8]) -> Result<Vec<u8>, String> {
    validate_input(pdf_bytes, "PDF")?;

    tracing::info!("Memulai PDF → Excel");

    let tables = match extract_pdf_words(pdf_bytes) {
        Ok(pages) => {
            tracing::info!(
                pages = pages.len(),
                "PDF text berhasil diekstrak menggunakan PDFium"
            );

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

    tracing::warn!("Struktur tabel tidak ditemukan menggunakan PDFium, menggunakan OCR fallback");

    pdf_to_excel_ocr(pdf_bytes)
}

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

    let mut tables = Vec::new();

    for (index, image_path) in image_paths.iter().enumerate() {
        if tables.len() >= MAX_OCR_TABLES {
            tracing::warn!(
                max_tables = MAX_OCR_TABLES,
                "Batas jumlah tabel OCR tercapai; pemrosesan dihentikan"
            );
            break;
        }

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
                continue;
            }
        };

        if items.is_empty() {
            tracing::warn!(page = index + 1, "Tesseract tidak menemukan text");
            continue;
        }

        let words = ocr_items_to_pdf_words(&items);

        tracing::debug!(
            page = index + 1,
            words = words.len(),
            "OCR page berhasil diproses ke PdfWord"
        );

        // Hanya satu halaman yang hidup di memory selama detection.
        let page_tables = detect_tables(std::slice::from_ref(&words));
        tables.extend(page_tables);
    }

    cleanup_ocr_files(&image_paths);

    if tables.is_empty() {
        return Err("OCR tidak menghasilkan struktur tabel yang valid.".to_owned());
    }

    tracing::info!(
        tables = tables.len(),
        "OCR berhasil mendeteksi struktur tabel"
    );

    build_workbook(&tables)
}

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

fn build_workbook(tables: &[Table]) -> Result<Vec<u8>, String> {
    let mut workbook = Workbook::new();
    let mut total_cells = 0usize;

    for (table_index, table) in tables.iter().enumerate() {
        let rows = &table.rows;

        let table_cells = rows
            .iter()
            .try_fold(0usize, |total, row| total.checked_add(row.len()))
            .ok_or_else(|| "Jumlah cell Excel terlalu besar".to_owned())?;

        total_cells = total_cells
            .checked_add(table_cells)
            .ok_or_else(|| "Jumlah cell Excel terlalu besar".to_owned())?;

        if total_cells > MAX_WORKBOOK_CELLS {
            return Err(format!(
                "Jumlah cell hasil Excel melebihi batas maksimum ({MAX_WORKBOOK_CELLS})"
            ));
        }

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

    let output = workbook
        .save_to_buffer()
        .map_err(|error| format!("Gagal membuat XLSX: {error}"))?;

    if output.len() > MAX_XLSX_OUTPUT_BYTES {
        return Err(format!(
            "XLSX hasil konversi melebihi batas maksimum ({} MiB)",
            MAX_XLSX_OUTPUT_BYTES / 1024 / 1024
        ));
    }

    Ok(output)
}

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

    for (column_index, width) in widths.iter().enumerate() {
        worksheet
            .set_column_width(
                column_index as u16,
                (*width).clamp(MIN_COLUMN_WIDTH, MAX_COLUMN_WIDTH),
            )
            .map_err(|error| format!("Gagal mengatur lebar kolom: {error}"))?;
    }

    worksheet
        .set_freeze_panes(actual_header_row as u32 + 1, 0)
        .map_err(|error| format!("Gagal mengatur freeze panes: {error}"))?;

    Ok(())
}

fn parse_number(value: &str) -> Option<f64> {
    let value = value.trim();

    if value.is_empty() {
        return None;
    }

    value.parse::<f64>().ok()
}
