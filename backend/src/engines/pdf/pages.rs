use lopdf::{Document, Object};

use super::common::validate_pdf;

/// Mengatur ulang halaman PDF.
///
/// `page_order` menggunakan nomor halaman berbasis 1.
///
/// Contoh:
///
/// ```text
/// PDF asli:
/// 1 2 3 4 5
///
/// page_order:
/// 5 3 1
///
/// hasil:
/// 5 3 1
/// ```
///
/// Nomor halaman yang sama boleh muncul lebih dari sekali,
/// sehingga operasi duplicate page juga didukung.
///
/// Halaman yang tidak ada di `page_order` akan dihapus
/// dari hasil akhir.
pub fn manage_pages(pdf_bytes: &[u8], page_order: &[u32]) -> Result<Vec<u8>, String> {
    validate_pdf(pdf_bytes)?;

    if page_order.is_empty() {
        return Err("Urutan halaman tidak boleh kosong".to_owned());
    }

    if page_order.contains(&0) {
        return Err("Nomor halaman harus dimulai dari 1".to_owned());
    }

    let mut document =
        Document::load_mem(pdf_bytes).map_err(|error| format!("Gagal membaca PDF: {error}"))?;

    let pages = document.get_pages();

    if pages.is_empty() {
        return Err("PDF tidak memiliki halaman.".to_owned());
    }

    /*
     * get_pages() menggunakan nomor halaman berbasis 1.
     *
     * Kita validasi seluruh page_order terlebih dahulu
     * sebelum memodifikasi document.
     */
    let mut selected_page_ids = Vec::with_capacity(page_order.len());

    for &page_number in page_order {
        let page_id = pages.get(&page_number).copied().ok_or_else(|| {
            format!(
                "Halaman {page_number} tidak ditemukan. \
                         PDF hanya memiliki {} halaman.",
                pages.len()
            )
        })?;

        selected_page_ids.push(page_id);
    }

    /*
     * Kita membangun document baru berdasarkan object
     * yang sudah ada.
     *
     * Dengan pendekatan ini kita tidak perlu memodifikasi
     * content stream halaman secara manual.
     */
    let mut output = Document::with_version(document.version.clone());

    /*
     * Object ID baru dimulai dari 1.
     *
     * Kita perlu melakukan renumbering agar object
     * yang direferensikan halaman tidak bentrok.
     */
    document.renumber_objects_with(1);

    /*
     * get_pages() harus diambil kembali setelah
     * renumbering karena ObjectId telah berubah.
     */
    let pages = document.get_pages();

    let mut selected_page_ids = Vec::with_capacity(page_order.len());

    for &page_number in page_order {
        let page_id = pages
            .get(&page_number)
            .copied()
            .ok_or_else(|| format!("Halaman {page_number} tidak ditemukan."))?;

        selected_page_ids.push(page_id);
    }

    /*
     * Ambil Catalog dan Pages root dari document asli.
     */
    let catalog_id = document
        .trailer
        .get(b"Root")
        .map_err(|error| format!("Catalog PDF tidak ditemukan: {error}"))?
        .as_reference()
        .map_err(|error| format!("Reference Catalog PDF tidak valid: {error}"))?;

    let catalog_object = document
        .get_object(catalog_id)
        .map_err(|error| format!("Gagal membaca Catalog PDF: {error}"))?
        .clone();

    let catalog_dictionary = catalog_object
        .as_dict()
        .map_err(|error| format!("Catalog PDF tidak valid: {error}"))?;

    let original_pages_id = catalog_dictionary
        .get(b"Pages")
        .map_err(|error| format!("Pages root tidak ditemukan: {error}"))?
        .as_reference()
        .map_err(|error| format!("Reference Pages PDF tidak valid: {error}"))?;

    let original_pages_object = document
        .get_object(original_pages_id)
        .map_err(|error| format!("Gagal membaca Pages root: {error}"))?
        .clone();

    let original_pages_dictionary = original_pages_object
        .as_dict()
        .map_err(|error| format!("Object Pages PDF tidak valid: {error}"))?;

    /*
     * Cari object ID baru untuk Pages root.
     *
     * Kita menggunakan object ID baru setelah semua
     * object dimasukkan ke output.
     */
    let mut next_id = document.max_id.saturating_add(1);

    let output_pages_id = (next_id, 0);

    next_id = next_id.saturating_add(1);

    let output_catalog_id = (next_id, 0);

    /*
     * Salin seluruh object dari document asli.
     *
     * Content stream, font, image, XObject, annotation,
     * dan dependency lain tetap dipertahankan.
     */
    output.objects = document.objects.clone();

    /*
     * Page object harus diproses ulang karena Parent-nya
     * harus menunjuk Pages root baru.
     *
     * Kita juga membuat daftar Kids berdasarkan
     * page_order, bukan ObjectId.
     */
    let mut kids = Vec::with_capacity(selected_page_ids.len());

    for page_id in selected_page_ids {
        let page_object = output
            .objects
            .get_mut(&page_id)
            .ok_or_else(|| format!("Object halaman {:?} tidak ditemukan.", page_id))?;

        let page_dictionary = page_object
            .as_dict_mut()
            .map_err(|error| format!("Object halaman PDF tidak valid: {error}"))?;

        page_dictionary.set("Parent", output_pages_id);

        kids.push(Object::Reference(page_id));
    }

    /*
     * Buat Pages dictionary baru.
     */
    let mut pages_dictionary = original_pages_dictionary.clone();

    pages_dictionary.remove(b"Parent");

    pages_dictionary.set("Type", "Pages");

    pages_dictionary.set("Count", page_order.len() as u32);

    pages_dictionary.set("Kids", kids);

    output
        .objects
        .insert(output_pages_id, Object::Dictionary(pages_dictionary));

    /*
     * Buat Catalog baru yang menunjuk Pages root baru.
     */
    let mut new_catalog = catalog_dictionary.clone();

    new_catalog.set("Type", "Catalog");

    new_catalog.set("Pages", output_pages_id);

    /*
     * Outline lama dibuang karena bookmark lama
     * dapat menunjuk halaman yang sudah dihapus atau
     * dipindahkan.
     */
    new_catalog.remove(b"Outlines");

    output
        .objects
        .insert(output_catalog_id, Object::Dictionary(new_catalog));

    /*
     * Root trailer harus menunjuk Catalog baru.
     */
    output.trailer.set("Root", output_catalog_id);

    /*
     * Hapus object Catalog dan Pages lama karena
     * keduanya sudah digantikan.
     */
    output.objects.remove(&catalog_id);

    output.objects.remove(&original_pages_id);

    /*
     * Rapikan object ID sebelum output.
     */
    output.renumber_objects();

    output.adjust_zero_pages();

    output.compress();

    let mut result = Vec::with_capacity(pdf_bytes.len());

    output
        .save_to(&mut result)
        .map_err(|error| format!("Gagal menyimpan PDF hasil pengaturan halaman: {error}"))?;

    if result.is_empty() {
        return Err("PDF hasil pengaturan halaman kosong.".to_owned());
    }

    Ok(result)
}
