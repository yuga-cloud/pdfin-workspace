use std::collections::BTreeMap;

use lopdf::{Document, Object, ObjectId};

use super::common::validate_pdf;

/// Menggabungkan beberapa PDF menjadi satu dokumen.
///
/// Urutan halaman dijaga berdasarkan urutan halaman asli
/// masing-masing PDF, bukan berdasarkan ObjectId.
///
/// Setiap dokumen di-renumber terlebih dahulu agar ObjectId
/// dari dokumen yang berbeda tidak saling bertabrakan.
pub fn merge_pdfs(pdfs: &[&[u8]]) -> Result<Vec<u8>, String> {
    if pdfs.is_empty() {
        return Err("Tidak ada file PDF yang akan digabungkan".to_owned());
    }

    for (index, pdf) in pdfs.iter().enumerate() {
        validate_pdf(pdf).map_err(|error| format!("PDF ke-{} tidak valid: {error}", index + 1))?;
    }

    let mut documents = Vec::with_capacity(pdfs.len());

    for (index, pdf) in pdfs.iter().enumerate() {
        let document = Document::load_mem(pdf)
            .map_err(|error| format!("Gagal membaca PDF ke-{}: {error}", index + 1))?;

        documents.push(document);
    }

    /*
     * Object ID awal untuk setiap dokumen.
     *
     * Dokumen pertama:
     *   1.x, 2.x, 3.x, ...
     *
     * Dokumen berikutnya dimulai setelah max_id
     * dokumen sebelumnya.
     */
    let mut next_object_id: u32 = 1;

    /*
     * Vec digunakan dengan sengaja.
     *
     * Jangan gunakan BTreeMap<ObjectId, ...> untuk halaman,
     * karena ObjectId tidak merepresentasikan nomor halaman.
     *
     * Vec mempertahankan:
     *
     * PDF 1:
     *   page 1
     *   page 2
     *   page 3
     *
     * lalu PDF 2:
     *   page 1
     *   page 2
     *
     * menjadi:
     *
     * 1 → 2 → 3 → 4 → 5
     */
    let mut pages_in_order: Vec<(ObjectId, Object)> = Vec::new();

    /*
     * Semua object selain Catalog, Pages, Page,
     * dan Outlines akan dikumpulkan di sini.
     *
     * BTreeMap tetap berguna di sini karena object ID
     * hanya digunakan sebagai key unik.
     */
    let mut document_objects: BTreeMap<ObjectId, Object> = BTreeMap::new();

    let mut output = Document::with_version("1.5");

    /*
     * Object Catalog dan Pages dari dokumen pertama
     * akan dijadikan root dokumen hasil merge.
     */
    let mut catalog_object: Option<(ObjectId, Object)> = None;

    let mut pages_object: Option<(ObjectId, Object)> = None;

    for mut document in documents {
        /*
         * Pisahkan object ID setiap source PDF.
         */
        document.renumber_objects_with(next_object_id);

        next_object_id = document.max_id.saturating_add(1);

        /*
         * get_pages() mengembalikan halaman berdasarkan
         * urutan page tree:
         *
         * 1 -> ObjectId halaman pertama
         * 2 -> ObjectId halaman kedua
         * dst.
         *
         * Kita iterasikan langsung berdasarkan nomor halaman
         * supaya urutan halaman tetap benar.
         */
        let pages = document.get_pages();

        for object_id in pages.values().copied() {
            let object = document
                .get_object(object_id)
                .map_err(|error| format!("Gagal membaca object halaman PDF: {error}"))?
                .clone();

            pages_in_order.push((object_id, object));
        }

        /*
         * Ambil seluruh object dari dokumen sumber.
         *
         * Object Page akan diproses ulang setelah semua
         * object terkumpul.
         */
        for (object_id, object) in document.objects {
            match object.type_name().unwrap_or_default() {
                b"Catalog" => {
                    /*
                     * Hanya Catalog pertama yang digunakan.
                     */
                    if catalog_object.is_none() {
                        catalog_object = Some((object_id, object));
                    }
                }

                b"Pages" => {
                    /*
                     * Hanya Pages pertama yang digunakan
                     * sebagai root page tree.
                     *
                     * Dictionary dari source pertama cukup
                     * menjadi dasar karena Kids dan Count
                     * akan kita bangun ulang di bawah.
                     */
                    if pages_object.is_none() {
                        pages_object = Some((object_id, object));
                    }
                }

                b"Page" => {
                    /*
                     * Page sudah disimpan dalam
                     * pages_in_order.
                     */
                }

                b"Outlines" | b"Outline" => {
                    /*
                     * Outline/bookmark dari dokumen sumber
                     * tidak dapat dipertahankan secara aman
                     * setelah object tree digabungkan.
                     *
                     * Karena itu kita buang.
                     */
                }

                _ => {
                    document_objects.insert(object_id, object);
                }
            }
        }
    }

    let (pages_id, pages_source_object) =
        pages_object.ok_or_else(|| "Object Pages tidak ditemukan pada PDF.".to_owned())?;

    let (catalog_id, catalog_source_object) =
        catalog_object.ok_or_else(|| "Object Catalog tidak ditemukan pada PDF.".to_owned())?;

    /*
     * Masukkan semua object non-page ke output.
     */
    output.objects.extend(document_objects);

    /*
     * Setiap Page harus menunjuk ke Pages root
     * yang baru.
     */
    for (object_id, object) in pages_in_order.iter() {
        let dictionary = object
            .as_dict()
            .map_err(|error| format!("Object halaman PDF tidak valid: {error}"))?;

        let mut dictionary = dictionary.clone();

        dictionary.set("Parent", pages_id);

        output
            .objects
            .insert(*object_id, Object::Dictionary(dictionary));
    }

    /*
     * Bangun ulang Pages dictionary.
     */
    let mut pages_dictionary = pages_source_object
        .as_dict()
        .map_err(|error| format!("Object Pages tidak valid: {error}"))?
        .clone();

    /*
     * Pastikan Pages object sendiri tidak
     * mempunyai parent.
     */
    pages_dictionary.remove(b"Parent");

    pages_dictionary.set("Type", "Pages");

    pages_dictionary.set("Count", pages_in_order.len() as u32);

    /*
     * PENTING:
     *
     * Kids dibuat dari Vec sehingga urutan:
     *
     * PDF A page 1
     * PDF A page 2
     * PDF B page 1
     * PDF B page 2
     *
     * tetap dipertahankan.
     */
    let kids = pages_in_order
        .iter()
        .map(|(object_id, _)| Object::Reference(*object_id))
        .collect::<Vec<_>>();

    pages_dictionary.set("Kids", kids);

    output
        .objects
        .insert(pages_id, Object::Dictionary(pages_dictionary));

    /*
     * Bangun ulang Catalog.
     */
    let mut catalog_dictionary = catalog_source_object
        .as_dict()
        .map_err(|error| format!("Object Catalog tidak valid: {error}"))?
        .clone();

    catalog_dictionary.set("Type", "Catalog");

    catalog_dictionary.set("Pages", pages_id);

    /*
     * Outline dari PDF sumber dibuang karena
     * reference-nya dapat menunjuk object yang
     * tidak lagi valid setelah merge.
     */
    catalog_dictionary.remove(b"Outlines");

    output
        .objects
        .insert(catalog_id, Object::Dictionary(catalog_dictionary));

    /*
     * Set Root Catalog.
     */
    output.trailer.set("Root", catalog_id);

    /*
     * max_id harus mencerminkan object ID maksimum,
     * bukan jumlah object.
     */
    output.max_id = output
        .objects
        .keys()
        .map(|(id, _generation)| *id)
        .max()
        .unwrap_or(0);

    /*
     * Rapikan object IDs setelah seluruh document
     * tree selesai dibangun.
     */
    output.renumber_objects();

    /*
     * Pastikan struktur page tree dengan zero-page
     * tidak menyebabkan document invalid.
     */
    output.adjust_zero_pages();

    /*
     * Compress stream sebelum disimpan.
     */
    output.compress();

    let mut result = Vec::new();

    output
        .save_to(&mut result)
        .map_err(|error| format!("Gagal menyimpan PDF hasil merge: {error}"))?;

    if result.is_empty() {
        return Err("PDF hasil merge kosong.".to_owned());
    }

    Ok(result)
}
