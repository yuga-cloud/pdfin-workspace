use lopdf::Document;

use super::common::validate_pdf;

const MAX_SPLIT_OUTPUTS: usize = 64;

/// Memisahkan PDF berdasarkan rentang halaman.
///
/// Validasi dilakukan sebelum operasi berat agar PDF besar tidak
/// diproses jika request sudah tidak valid.
pub fn split_pdf(pdf_bytes: &[u8], ranges: &[(u32, u32)]) -> Result<Vec<Vec<u8>>, String> {
    validate_pdf(pdf_bytes)?;

    if ranges.is_empty() {
        return Err("Tidak ada rentang halaman yang dipilih".to_owned());
    }

    if ranges.len() > MAX_SPLIT_OUTPUTS {
        return Err(format!(
            "Jumlah output split melebihi batas maksimum ({MAX_SPLIT_OUTPUTS})"
        ));
    }

    let source =
        Document::load_mem(pdf_bytes).map_err(|error| format!("Gagal membaca PDF: {error}"))?;

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
    }

    let mut outputs = Vec::with_capacity(ranges.len());

    for &(start, end) in ranges {
        let mut document = source.clone();
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
        document.save_to(&mut output).map_err(|error| {
            format!("Gagal menyimpan hasil split halaman {start}-{end}: {error}")
        })?;

        outputs.push(output);
    }

    Ok(outputs)
}
