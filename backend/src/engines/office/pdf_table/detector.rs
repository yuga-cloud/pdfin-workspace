use std::cmp::Ordering;

use super::model::{PdfRow, PdfWord, Table};

const ROW_TOLERANCE: f32 = 5.0;

const MIN_TABLE_ROWS: usize = 2;
const MIN_TABLE_COLUMNS: usize = 2;

const COLUMN_TOLERANCE_MIN: f32 = 8.0;
const COLUMN_TOLERANCE_MAX: f32 = 24.0;

// Diturunkan menjadi 10% agar kolom dengan sel kosong tetap bisa menjadi anchor yang valid
const COLUMN_SUPPORT_RATIO: f32 = 0.10;

/* -------------------------------------------------------------------------- */
/* PUBLIC API                                                                 */
/* -------------------------------------------------------------------------- */

pub fn detect_tables(pages: &[Vec<PdfWord>]) -> Vec<Table> {
    let mut tables = Vec::new();

    for (page_index, words) in pages.iter().enumerate() {
        if words.is_empty() {
            continue;
        }

        /*
         * 1. Kelompokkan word berdasarkan posisi Y dan gabungkan kata yang berdekatan.
         */
        let rows = group_rows(words);

        if rows.len() < MIN_TABLE_ROWS {
            continue;
        }

        /*
         * 2. Cari region yang kemungkinan merupakan tabel.
         */
        let table_rows = find_table_region(&rows);

        if table_rows.len() < MIN_TABLE_ROWS {
            tracing::debug!(
                page = page_index + 1,
                rows = rows.len(),
                "Tidak ditemukan region tabel"
            );

            continue;
        }

        /*
         * 3. Bangun struktur tabel berdasarkan posisi horizontal word.
         */
        let table = build_table(&table_rows);

        if table.column_count() < MIN_TABLE_COLUMNS {
            tracing::debug!(
                page = page_index + 1,
                columns = table.column_count(),
                "Region tidak cukup kuat untuk dianggap tabel"
            );

            continue;
        }

        if table.is_empty() {
            continue;
        }

        tracing::info!(
            page = page_index + 1,
            rows = table.rows.len(),
            columns = table.column_count(),
            "Tabel PDF berhasil dideteksi"
        );

        tables.push(table);
    }

    tables
}

/* -------------------------------------------------------------------------- */
/* ROW DETECTION & CELL MERGING                                               */
/* -------------------------------------------------------------------------- */

fn group_rows(words: &[PdfWord]) -> Vec<PdfRow> {
    let mut sorted = words.to_vec();

    sorted.sort_unstable_by(|a, b| {
        a.top
            .partial_cmp(&b.top)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.left.partial_cmp(&b.left).unwrap_or(Ordering::Equal))
    });

    let mut rows: Vec<PdfRow> = Vec::new();

    for word in sorted {
        let Some(last) = rows.last_mut() else {
            rows.push(PdfRow {
                top: word.top,
                bottom: word.bottom,
                words: vec![word],
            });

            continue;
        };

        let row_height = last.height().max(word.height()).max(1.0);

        let tolerance = ROW_TOLERANCE.max(row_height * 0.35).min(10.0);

        let same_row = (last.top - word.top).abs() <= tolerance;

        if same_row {
            last.top = last.top.min(word.top);
            last.bottom = last.bottom.max(word.bottom);
            last.words.push(word);
        } else {
            rows.push(PdfRow {
                top: word.top,
                bottom: word.bottom,
                words: vec![word],
            });
        }
    }

    // Urutkan kata dari kiri ke kanan untuk setiap baris
    for row in &mut rows {
        row.words
            .sort_unstable_by(|a, b| a.left.partial_cmp(&b.left).unwrap_or(Ordering::Equal));
    }

    /*
     * TAHAP BARU: CELL MERGING (Penggabungan Kata Horizontal)
     * Menggabungkan kata-kata yang terpecah (seperti "2026", "-", "03")
     * menjadi satu sel/kolom yang utuh ("2026 - 03") sebelum kolom dideteksi.
     */
    let median_height = median(words.iter().map(|w| w.height()).collect()).max(1.0);

    // Tolerance dinamis: kata yang jaraknya lebih kecil dari ini akan disatukan.
    let merge_tolerance = (median_height * 1.2).clamp(5.0, 24.0);

    for row in &mut rows {
        if row.words.is_empty() {
            continue;
        }

        let mut merged_words = Vec::with_capacity(row.words.len());
        let mut current_word = row.words[0].clone();

        for word in row.words.iter().skip(1) {
            let horizontal_gap = word.left - current_word.right;

            // Jika jaraknya wajar sebagai spasi dalam satu kalimat/sel
            if horizontal_gap <= merge_tolerance {
                current_word.text.push(' ');
                current_word.text.push_str(&word.text);

                // Perluas Bounding Box agar mencakup seluruh teks
                current_word.right = current_word.right.max(word.right);
                current_word.top = current_word.top.min(word.top);
                current_word.bottom = current_word.bottom.max(word.bottom);
                current_word.center_x = (current_word.left + current_word.right) / 2.0;
                current_word.center_y = (current_word.top + current_word.bottom) / 2.0;
            } else {
                merged_words.push(current_word);
                current_word = word.clone();
            }
        }
        merged_words.push(current_word);
        row.words = merged_words;
    }

    rows
}

/* -------------------------------------------------------------------------- */
/* TABLE REGION                                                               */
/* -------------------------------------------------------------------------- */

fn find_table_region(rows: &[PdfRow]) -> Vec<PdfRow> {
    if rows.len() < MIN_TABLE_ROWS {
        return Vec::new();
    }

    let max_density = rows.iter().map(|row| row.words.len()).max().unwrap_or(0);

    if max_density < MIN_TABLE_COLUMNS {
        return Vec::new();
    }

    let density_threshold = (max_density as f32 * 0.5)
        .ceil()
        .max(MIN_TABLE_COLUMNS as f32) as usize;

    let mut best_start: Option<usize> = None;
    let mut best_end: Option<usize> = None;
    let mut best_score = 0usize;

    let mut current_start: Option<usize> = None;
    let mut current_score = 0usize;

    for (index, row) in rows.iter().enumerate() {
        let density = row.words.len();

        if density >= density_threshold {
            if current_start.is_none() {
                current_start = Some(index);
                current_score = 0;
            }

            current_score += density;

            let start = current_start.unwrap();

            if current_score > best_score {
                best_score = current_score;
                best_start = Some(start);
                best_end = Some(index);
            }
        } else if current_start.is_some() {
            let previous_density = rows[index.saturating_sub(1)].words.len();

            if previous_density >= MIN_TABLE_COLUMNS {
                continue;
            }

            current_start = None;
            current_score = 0;
        }
    }

    let (Some(start), Some(end)) = (best_start, best_end) else {
        return Vec::new();
    };

    let region = &rows[start..=end];

    if region.len() < MIN_TABLE_ROWS {
        return Vec::new();
    }

    region.to_vec()
}

/* -------------------------------------------------------------------------- */
/* TABLE BUILD                                                                */
/* -------------------------------------------------------------------------- */

fn build_table(rows: &[PdfRow]) -> Table {
    if rows.is_empty() {
        return Table { rows: Vec::new() };
    }

    let anchors = detect_columns(rows);

    if anchors.len() < MIN_TABLE_COLUMNS {
        return Table { rows: Vec::new() };
    }

    tracing::debug!(
        anchors = ?anchors,
        "Column anchors terdeteksi"
    );

    let mut result = Vec::with_capacity(rows.len());

    for row in rows {
        let mut cells = vec![String::new(); anchors.len()];

        for word in &row.words {
            let Some(column) = nearest_column(&anchors, word.left) else {
                continue;
            };

            if cells[column].is_empty() {
                cells[column] = word.text.clone();
            } else {
                cells[column].push(' ');
                cells[column].push_str(&word.text);
            }
        }

        while cells.last().is_some_and(|cell| cell.is_empty()) {
            cells.pop();
        }

        if cells.iter().any(|cell| !cell.is_empty()) {
            result.push(cells);
        }
    }

    normalize_rows(&mut result);

    Table { rows: result }
}

/* -------------------------------------------------------------------------- */
/* COLUMN DETECTION                                                           */
/* -------------------------------------------------------------------------- */

fn detect_columns(rows: &[PdfRow]) -> Vec<f32> {
    if rows.is_empty() {
        return Vec::new();
    }

    let median_height = median(
        rows.iter()
            .flat_map(|row| row.words.iter().map(|w| w.height()))
            .collect(),
    )
    .max(1.0);

    let tolerance = (median_height * 1.5).clamp(COLUMN_TOLERANCE_MIN, COLUMN_TOLERANCE_MAX);

    /*
     * Kumpulkan semua koordinat X dari seluruh baris,
     * sehingga header tabel yang berantakan tidak mengacaukan struktur kolom utama.
     */
    let mut all_x: Vec<f32> = rows
        .iter()
        .flat_map(|row| row.words.iter().map(|w| w.left))
        .collect();

    all_x.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    let mut clustered: Vec<f32> = Vec::new();
    let mut cluster_counts: Vec<usize> = Vec::new();

    for x in all_x {
        let mut found = false;

        if let Some(last) = clustered.last_mut()
            && (x - *last).abs() <= tolerance
        {
            let count = cluster_counts.last_mut().unwrap();
            *last = ((*last * *count as f32) + x) / (*count as f32 + 1.0);
            *count += 1;
            found = true;
        }

        if !found {
            clustered.push(x);
            cluster_counts.push(1);
        }
    }

    let minimum_support = ((rows.len() as f32) * COLUMN_SUPPORT_RATIO).ceil().max(2.0) as usize;

    let mut final_anchors: Vec<f32> = clustered
        .into_iter()
        .filter(|anchor| {
            let support = rows
                .iter()
                .filter(|row| {
                    row.words
                        .iter()
                        .any(|word| (word.left - *anchor).abs() <= tolerance * 2.0)
                })
                .count();

            support >= minimum_support
        })
        .collect();

    if final_anchors.len() < MIN_TABLE_COLUMNS {
        tracing::warn!("Support ratio filtering terlalu agresif, melakukan fallback.");
        if let Some(reference_row) = rows.iter().max_by_key(|row| row.words.len()) {
            final_anchors = reference_row.words.iter().map(|word| word.left).collect();
            final_anchors.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

            let mut deduped: Vec<f32> = Vec::new();
            for position in final_anchors {
                if let Some(last) = deduped.last_mut()
                    && (position - *last).abs() <= tolerance
                {
                    *last = (*last + position) / 2.0;
                    continue;
                }
                deduped.push(position);
            }
            final_anchors = deduped;
        }
    }

    final_anchors
}

/* -------------------------------------------------------------------------- */
/* COLUMN ASSIGNMENT                                                          */
/* -------------------------------------------------------------------------- */

fn nearest_column(anchors: &[f32], x: f32) -> Option<usize> {
    anchors
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            (*a - x)
                .abs()
                .partial_cmp(&(*b - x).abs())
                .unwrap_or(Ordering::Equal)
        })
        .map(|(index, _)| index)
}

/* -------------------------------------------------------------------------- */
/* NORMALIZATION                                                              */
/* -------------------------------------------------------------------------- */

fn normalize_rows(rows: &mut [Vec<String>]) {
    let column_count = rows.iter().map(Vec::len).max().unwrap_or(0);

    for row in rows {
        row.resize(column_count, String::new());
    }
}

/* -------------------------------------------------------------------------- */
/* STATISTICS                                                                 */
/* -------------------------------------------------------------------------- */

fn median(mut values: Vec<f32>) -> f32 {
    if values.is_empty() {
        return 0.0;
    }

    values.retain(|value| value.is_finite());

    if values.is_empty() {
        return 0.0;
    }

    values.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    let middle = values.len() / 2;

    if values.len().is_multiple_of(2) {
        (values[middle - 1] + values[middle]) / 2.0
    } else {
        values[middle]
    }
}
