use lopdf::{
    Document, Object, Stream,
    content::{Content, Operation},
    dictionary, xobject,
};

use crate::engines::common::validate_input;

const MAX_IMAGES_PER_DOCUMENT: usize = 64;
const MAX_TOTAL_INPUT_SIZE: usize = 256 * 1024 * 1024;
const MAX_IMAGE_DIMENSION: u32 = 20_000;
const MAX_IMAGE_PIXELS: u64 = 50_000_000;
const MAX_OUTPUT_SIZE_BYTES: usize = 256 * 1024 * 1024;

#[expect(
    dead_code,
    reason = "engine siap dipakai saat route JPG to PDF diaktifkan"
)]
pub fn jpg_to_pdf(image_bytes: &[u8]) -> Result<Vec<u8>, String> {
    jpgs_to_pdf(&[image_bytes])
}

pub fn jpgs_to_pdf(images: &[&[u8]]) -> Result<Vec<u8>, String> {
    if images.is_empty() {
        return Err("Tidak ada gambar yang akan dikonversi".to_owned());
    }

    if images.len() > MAX_IMAGES_PER_DOCUMENT {
        return Err(format!(
            "Jumlah gambar melebihi batas maksimal ({MAX_IMAGES_PER_DOCUMENT})"
        ));
    }

    let total_size = images
        .iter()
        .try_fold(0usize, |total, image| total.checked_add(image.len()));
    let total_size = total_size
        .ok_or_else(|| "Ukuran total gambar melebihi batas numerik yang didukung".to_owned())?;

    if total_size > MAX_TOTAL_INPUT_SIZE {
        return Err(format!(
            "Ukuran total gambar melebihi batas maksimal {} MB",
            MAX_TOTAL_INPUT_SIZE / 1024 / 1024
        ));
    }

    let mut document = Document::with_version("1.7");
    let pages_id = document.new_object_id();
    let mut page_ids = Vec::with_capacity(images.len());

    for (index, image_bytes) in images.iter().enumerate() {
        validate_input(image_bytes, "JPG")?;
        validate_jpeg_dimensions(image_bytes, index + 1)?;

        let image = xobject::image_from((*image_bytes).to_vec())
            .map_err(|error| format!("Gagal membaca gambar JPEG {}: {error}", index + 1))?;

        let width = image
            .dict
            .get(b"Width")
            .map_err(|error| format!("Dimensi width JPEG {} tidak tersedia: {error}", index + 1))?
            .as_i64()
            .map_err(|error| format!("Width JPEG {} tidak valid: {error}", index + 1))?;

        let height = image
            .dict
            .get(b"Height")
            .map_err(|error| format!("Dimensi height JPEG {} tidak tersedia: {error}", index + 1))?
            .as_i64()
            .map_err(|error| format!("Height JPEG {} tidak valid: {error}", index + 1))?;

        if width <= 0 || height <= 0 {
            return Err(format!("Dimensi JPEG {} tidak valid", index + 1));
        }

        if width as u32 > MAX_IMAGE_DIMENSION || height as u32 > MAX_IMAGE_DIMENSION {
            return Err(format!(
                "Resolusi JPEG {} terlalu besar: {}x{}",
                index + 1,
                width,
                height
            ));
        }

        let pixels = u64::try_from(width)
            .ok()
            .and_then(|width| u64::try_from(height).ok().and_then(|height| width.checked_mul(height)))
            .ok_or_else(|| format!("Dimensi JPEG {} terlalu besar", index + 1))?;

        if pixels > MAX_IMAGE_PIXELS {
            return Err(format!(
                "Jumlah pixel JPEG {} melebihi batas maksimum ({} juta pixel)",
                index + 1,
                MAX_IMAGE_PIXELS / 1_000_000
            ));
        }

        let image_id = document.add_object(image);
        let image_name = format!("Im{}", index + 1).into_bytes();

        let content = Content {
            operations: vec![
                Operation::new("q", vec![]),
                Operation::new(
                    "cm",
                    vec![
                        width.into(),
                        0.into(),
                        0.into(),
                        height.into(),
                        0.into(),
                        0.into(),
                    ],
                ),
                Operation::new("Do", vec![Object::Name(image_name.clone())]),
                Operation::new("Q", vec![]),
            ],
        };

        let content_id = document.add_object(Stream::new(
            dictionary! {},
            content
                .encode()
                .map_err(|error| format!("Gagal membuat content stream PDF: {error}"))?,
        ));

        let page_id = document.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), width.into(), height.into()],
            "Contents" => content_id,
        });

        document
            .add_xobject(page_id, image_name, image_id)
            .map_err(|error| {
                format!("Gagal memasang JPEG {} ke halaman PDF: {error}", index + 1)
            })?;

        page_ids.push(page_id);
    }

    document.objects.insert(
        pages_id,
        dictionary! {
            "Type" => "Pages",
            "Count" => page_ids.len() as u32,
            "Kids" => page_ids.iter().copied().map(Object::Reference).collect::<Vec<_>>(),
        }
        .into(),
    );

    let catalog_id = document.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });

    document.trailer.set("Root", catalog_id);
    document.compress();

    let mut output = Vec::new();

    document
        .save_to(&mut output)
        .map_err(|error| format!("Gagal menyimpan PDF: {error}"))?;

    if output.is_empty() {
        return Err("PDF hasil konversi kosong".to_owned());
    }

    if output.len() > MAX_OUTPUT_SIZE_BYTES {
        return Err(format!(
            "PDF hasil konversi melebihi batas maksimum ({} MiB)",
            MAX_OUTPUT_SIZE_BYTES / 1024 / 1024
        ));
    }

    Ok(output)
}

fn validate_jpeg_dimensions(bytes: &[u8], image_number: usize) -> Result<(), String> {
    if bytes.len() < 4 || bytes[..2] != [0xFF, 0xD8] {
        return Err(format!("File JPEG {} tidak valid", image_number));
    }

    let mut offset = 2usize;

    while offset < bytes.len() {
        while offset < bytes.len() && bytes[offset] == 0xFF {
            offset += 1;
        }

        if offset >= bytes.len() {
            break;
        }

        let marker = bytes[offset];
        offset += 1;

        match marker {
            0xD9 => break,
            0xD8 | 0x01 | 0xD0..=0xD7 => continue,
            _ => {}
        }

        if offset + 2 > bytes.len() {
            return Err(format!("Struktur JPEG {} terpotong", image_number));
        }

        let segment_length = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as usize;

        if segment_length < 2 || offset + segment_length > bytes.len() {
            return Err(format!("Struktur JPEG {} tidak valid", image_number));
        }

        if is_sof_marker(marker) {
            if segment_length < 7 {
                return Err(format!(
                    "Metadata dimensi JPEG {} tidak lengkap",
                    image_number
                ));
            }

            let data_start = offset + 2;
            let height = u16::from_be_bytes([bytes[data_start + 1], bytes[data_start + 2]]) as u32;
            let width = u16::from_be_bytes([bytes[data_start + 3], bytes[data_start + 4]]) as u32;

            if width == 0 || height == 0 {
                return Err(format!("Dimensi JPEG {} tidak valid", image_number));
            }

            if width > MAX_IMAGE_DIMENSION || height > MAX_IMAGE_DIMENSION {
                return Err(format!(
                    "Resolusi JPEG {} terlalu besar: {}x{}",
                    image_number, width, height
                ));
            }

            return Ok(());
        }

        offset += segment_length;
    }

    Err(format!("Dimensi JPEG {} tidak ditemukan", image_number))
}

fn is_sof_marker(marker: u8) -> bool {
    matches!(
        marker,
        0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_jpeg_before_decode() {
        let error = validate_jpeg_dimensions(b"PK\x03\x04", 1).unwrap_err();
        assert!(error.contains("tidak valid"));
    }

    #[test]
    fn rejects_truncated_jpeg_segment() {
        let error = validate_jpeg_dimensions(&[0xFF, 0xD8, 0xFF, 0xC0, 0x00], 1).unwrap_err();
        assert!(error.contains("terpotong"));
    }

    #[test]
    fn accepts_jpeg_with_valid_sof_dimensions() {
        let jpeg = [
            0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x07, 0xD0, 0x0F, 0xA0, 0x03, 0x00, 0x11,
            0x00, 0x22, 0x00, 0x33,
        ];

        assert!(validate_jpeg_dimensions(&jpeg, 1).is_ok());
    }

    #[test]
    fn rejects_jpeg_over_dimension_limit() {
        let jpeg = [
            0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x4E, 0x20, 0x4E, 0x21, 0x03, 0x00, 0x11,
            0x00, 0x22, 0x00, 0x33,
        ];

        let error = validate_jpeg_dimensions(&jpeg, 1).unwrap_err();
        assert!(error.contains("terlalu besar"));
    }

    #[test]
    fn rejects_jpeg_over_pixel_limit() {
        let jpeg = [
            0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x4E, 0x20, 0x30, 0xD4, 0x03, 0x00,
            0x11, 0x00, 0x22, 0x00, 0x33,
        ];

        let error = validate_jpeg_dimensions(&jpeg, 1).unwrap_err();
        assert!(error.contains("terlalu besar") || error.contains("tidak valid"));
    }
}
