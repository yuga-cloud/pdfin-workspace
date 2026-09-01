use std::path::Path;

use lopdf::Document;

use super::common::{load_pdf_document_from_path, validate_pdf_path};

const MAX_SPLIT_OUTPUTS: usize = 64;
const MAX_SPLIT_RANGE_PAGES: u32 = 10_000;
const MAX_SPLIT_TOTAL_OUTPUT_BYTES: usize = 1024 * 1024 * 1024;

/// Memisahkan PDF berdasarkan rentang halaman.
///
/// Validasi dilakukan sebelum operasi berat agar PDF besar tidak
/// diproses jika request sudah tidak valid.
pub fn split_pdf_from_path(pdf_path: &Path, ranges: &[(u32, u32)]) -> Result<Vec<Vec<u8>>, String> {
    validate_pdf_path(pdf_path)?;

    if ranges.is_empty() {
        return Err("Tidak ada rentang halaman yang dipilih".to_owned());
    }

    if ranges.len() > MAX_SPLIT_OUTPUTS {
        return Err(format!(
            "Jumlah output split melebihi batas maksimum ({MAX_SPLIT_OUTPUTS})"
        ));
    }

    let mut source = load_pdf_document_from_path(pdf_path)?;

    let page_count = source.get_pages().len() as u32;

    if page_count == 0 {
        return Err("PDF tidak memiliki halaman".to_owned());
    }

    for &(start, end) in ranges {
        if start == 0 || end == 0 {
            return Err("Nomor halaman harus dimulai dari 1".to_owned());
        }

        if start > end {
            return Err(format!("Rentang halaman tidak valid: {start}-{end}"));
        }

        if end > page_count {
            return Err(format!(
                "Halaman {end} melebihi jumlah halaman PDF ({page_count})"
            ));
        }

        if end - start + 1 > MAX_SPLIT_RANGE_PAGES {
            return Err(format!(
                "Rentang split melebihi batas maksimum ({MAX_SPLIT_RANGE_PAGES} halaman)"
            ));
        }
    }

    if ranges.len() == 1 {
        let (start, end) = ranges[0];
        let output = split_document_output(&mut source, start, end)?;
        return Ok(vec![output]);
    }

    let mut outputs = Vec::with_capacity(ranges.len());
    let mut total_output_bytes = 0usize;

    for &(start, end) in ranges {
        let mut document = source.clone();
        let output = split_document_output(&mut document, start, end)?;

        total_output_bytes = total_output_bytes
            .checked_add(output.len())
            .ok_or_else(|| "Ukuran total hasil split terlalu besar".to_owned())?;

        if total_output_bytes > MAX_SPLIT_TOTAL_OUTPUT_BYTES {
            return Err(format!(
                "Ukuran total hasil split melebihi batas maksimum ({} MiB)",
                MAX_SPLIT_TOTAL_OUTPUT_BYTES / 1024 / 1024
            ));
        }

        outputs.push(output);
    }

    Ok(outputs)
}

fn split_document_output(document: &mut Document, start: u32, end: u32) -> Result<Vec<u8>, String> {
    let pages = document.get_pages();

    let pages_to_delete = pages
        .keys()
        .copied()
        .filter(|page| *page < start || *page > end)
        .collect::<Vec<_>>();

    if !pages_to_delete.is_empty() {
        document.delete_pages(&pages_to_delete);
    }

    document.prune_objects();
    document.renumber_objects();

    let mut output = Vec::new();
    document
        .save_to(&mut output)
        .map_err(|error| format!("Gagal menyimpan hasil split halaman {start}-{end}: {error}"))?;

    if output.is_empty() {
        return Err(format!("Hasil split halaman {start}-{end} kosong"));
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_ranges() {
        let result = split_pdf_from_path(Path::new("missing.pdf"), &[]);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_more_than_max_outputs() {
        let ranges = vec![(1_u32, 1_u32); MAX_SPLIT_OUTPUTS + 1];
        let result = split_pdf_from_path(Path::new("missing.pdf"), &ranges);
        assert!(result.is_err());
    }
}
