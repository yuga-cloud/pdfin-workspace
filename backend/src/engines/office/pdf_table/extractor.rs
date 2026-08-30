use super::model::PdfWord;

const WORD_GAP_FACTOR: f32 = 0.55;
const MIN_WORD_GAP: f32 = 1.5;
const MAX_WORD_GAP: f32 = 12.0;
const ROW_TOLERANCE_FACTOR: f32 = 0.45;
const MIN_ROW_TOLERANCE: f32 = 2.0;
const MAX_ROW_TOLERANCE: f32 = 8.0;

const MAX_EXTRACTION_PAGES: usize = 5_000;
const MAX_WORDS_PER_PAGE: usize = 50_000;

pub fn extract_pdf_words(pdf_bytes: &[u8]) -> Result<Vec<Vec<PdfWord>>, String> {
    let pdfium = pdfium_bundled::bind_pdfium_silent()
        .map_err(|error| format!("Gagal memuat PDFium: {error}"))?;

    let document = pdfium
        .load_pdf_from_byte_slice(pdf_bytes, None)
        .map_err(|error| format!("Gagal membuka PDF dengan PDFium: {error}"))?;

    let document_page_count = document.pages().len() as usize;

    if document_page_count > MAX_EXTRACTION_PAGES {
        return Err(format!(
            "Jumlah halaman PDF melebihi batas ekstraksi ({MAX_EXTRACTION_PAGES})"
        ));
    }

    let mut pages = Vec::with_capacity(document_page_count);

    for (page_index, page) in document.pages().iter().enumerate() {
        let text = page.text().map_err(|error| {
            format!(
                "Gagal membaca text PDF pada halaman {}: {error}",
                page_index + 1
            )
        })?;

        if text.is_empty() {
            pages.push(Vec::new());
            continue;
        }

        /*
         * PENTING: koordinat native PDFium memakai origin kiri-bawah
         * dengan sumbu Y mengarah ke ATAS halaman. Artinya
         * `bounds.top()` justru bernilai LEBIH BESAR daripada
         * `bounds.bottom()` (kebalikan dari koordinat layar/screen
         * yang dipakai di seluruh kode ini, termasuk PdfWord::height()
         * di model.rs dan pengurutan baris berdasarkan `top` di
         * detector.rs).
         *
         * Tanpa konversi ini:
         * - PdfWord::height() selalu clamp ke 0.0 (bottom - top < 0),
         *   sehingga semua toleransi adaptif (row/word gap) collapse
         *   ke nilai minimum, tidak peduli ukuran font sebenarnya.
         * - Baris/kata diurutkan dari BAWAH ke ATAS halaman, sehingga
         *   output tabel/baris jadi terbalik.
         *
         * Kita konversi ke koordinat top-down (Y=0 di atas halaman)
         * di sini, sekali per halaman, supaya seluruh kode di bawahnya
         * (extractor + detector + model) bisa konsisten menganggap
         * top < bottom seperti koordinat layar biasa.
         */
        // NOTE: sesuaikan pemanggilan ini dengan API pdfium-render versi
        // yang lo pakai kalau nama methodnya beda — intinya kita cuma
        // butuh tinggi halaman dalam PdfPoints. Di versi pdfium-render
        // yang umum dipakai sekarang ini tersedia lewat `page.page_size()`.
        let page_height = page.page_size().height().value;

        let chars = text.chars();

        let mut words = Vec::new();

        let mut current_text = String::new();

        let mut current_left = 0.0_f32;
        let mut current_right = 0.0_f32;
        let mut current_top = 0.0_f32;
        let mut current_bottom = 0.0_f32;

        let mut previous_right: Option<f32> = None;
        let mut previous_top: Option<f32> = None;
        let mut previous_height: Option<f32> = None;

        for character in chars.iter() {
            let Some(value) = character.unicode_char() else {
                continue;
            };

            let bounds = match character.tight_bounds() {
                Ok(bounds) => bounds,
                Err(_) => {
                    /*
                     * Bounds gagal diambil (mis. karakter kontrol/
                     * invisible). Sebelumnya karakter ini di-skip total
                     * dengan `continue`, artinya HILANG dari teks yang
                     * diekstrak tanpa jejak. Kita tetap masukkan
                     * karakternya ke word yang sedang berjalan supaya
                     * teksnya tidak diam-diam corrupt, walau posisinya
                     * tidak ikut memperluas bounding box.
                     */
                    if !current_text.is_empty() {
                        current_text.push(value);
                    }

                    continue;
                }
            };

            let left = bounds.left().value;
            let right = bounds.right().value;

            // Konversi dari koordinat native PDFium (Y ke atas, origin
            // kiri-bawah) ke koordinat top-down (Y ke bawah, origin
            // kiri-atas) supaya `top < bottom` konsisten dengan asumsi
            // di model.rs dan detector.rs.
            let top = page_height - bounds.top().value;
            let bottom = page_height - bounds.bottom().value;

            let height = (bottom - top).abs().max(0.1);

            /*
             * PDFium bisa mengembalikan whitespace dan newline
             * sebagai character.
             */
            if value.is_whitespace() {
                flush_word(
                    &mut words,
                    &mut current_text,
                    &mut current_left,
                    &mut current_right,
                    &mut current_top,
                    &mut current_bottom,
                );

                previous_right = None;
                previous_top = None;
                previous_height = None;

                continue;
            }

            /*
             * Kalau karakter sebelumnya tidak ada,
             * langsung mulai word baru.
             */
            let Some(previous_right_value) = previous_right else {
                current_text.push(value);

                current_left = left;
                current_right = right;
                current_top = top;
                current_bottom = bottom;

                previous_right = Some(right);
                previous_top = Some(top);
                previous_height = Some(height);

                continue;
            };

            let previous_height_value = previous_height.unwrap_or(height);

            /*
             * Tolerance vertikal dibuat relatif terhadap ukuran font.
             */
            let row_tolerance = (previous_height_value * ROW_TOLERANCE_FACTOR)
                .clamp(MIN_ROW_TOLERANCE, MAX_ROW_TOLERANCE);

            let same_line = previous_top
                .map(|previous| (previous - top).abs() <= row_tolerance)
                .unwrap_or(true);

            let horizontal_gap = left - previous_right_value;

            /*
             * Jangan memakai angka fixed 5 px.
             *
             * Ukuran font PDF bisa berbeda jauh.
             */
            let word_gap =
                (previous_height_value * WORD_GAP_FACTOR).clamp(MIN_WORD_GAP, MAX_WORD_GAP);

            let new_word = !same_line || horizontal_gap > word_gap;

            if new_word {
                flush_word(
                    &mut words,
                    &mut current_text,
                    &mut current_left,
                    &mut current_right,
                    &mut current_top,
                    &mut current_bottom,
                );

                current_text.push(value);

                current_left = left;
                current_right = right;
                current_top = top;
                current_bottom = bottom;
            } else {
                current_text.push(value);

                current_right = current_right.max(right);
                current_top = current_top.min(top);
                current_bottom = current_bottom.max(bottom);
            }

            if words.len() >= MAX_WORDS_PER_PAGE {
                return Err(format!(
                    "Jumlah word pada halaman {} melebihi batas maksimum ({MAX_WORDS_PER_PAGE})",
                    page_index + 1
                ));
            }

            previous_right = Some(right);
            previous_top = Some(top);
            previous_height = Some(height);
        }

        flush_word(
            &mut words,
            &mut current_text,
            &mut current_left,
            &mut current_right,
            &mut current_top,
            &mut current_bottom,
        );

        if words.len() > MAX_WORDS_PER_PAGE {
            return Err(format!(
                "Jumlah word pada halaman {} melebihi batas maksimum ({MAX_WORDS_PER_PAGE})",
                page_index + 1
            ));
        }

        tracing::debug!(
            page = page_index + 1,
            words = words.len(),
            "PDFium text extraction selesai"
        );

        pages.push(words);
    }

    Ok(pages)
}

fn flush_word(
    words: &mut Vec<PdfWord>,
    text: &mut String,
    left: &mut f32,
    right: &mut f32,
    top: &mut f32,
    bottom: &mut f32,
) {
    let value = text.trim();

    if !value.is_empty() {
        words.push(PdfWord::new(value.to_owned(), *left, *right, *top, *bottom));
    }

    text.clear();

    *left = 0.0;
    *right = 0.0;
    *top = 0.0;
    *bottom = 0.0;
}
