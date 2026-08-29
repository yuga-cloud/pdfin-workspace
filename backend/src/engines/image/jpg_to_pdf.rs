use lopdf::{
    Document, Object, Stream,
    content::{Content, Operation},
    dictionary, xobject,
};

use crate::engines::common::validate_input;

pub fn jpg_to_pdf(image_bytes: &[u8]) -> Result<Vec<u8>, String> {
    validate_input(image_bytes, "JPG")?;

    let image = xobject::image_from(image_bytes.to_vec())
        .map_err(|error| format!("Gagal membaca gambar JPEG: {error}"))?;

    let width = image
        .dict
        .get(b"Width")
        .map_err(|error| format!("Dimensi width JPEG tidak tersedia: {error}"))?
        .as_i64()
        .map_err(|error| format!("Width JPEG tidak valid: {error}"))?;

    let height = image
        .dict
        .get(b"Height")
        .map_err(|error| format!("Dimensi height JPEG tidak tersedia: {error}"))?
        .as_i64()
        .map_err(|error| format!("Height JPEG tidak valid: {error}"))?;

    if width <= 0 || height <= 0 {
        return Err("Dimensi JPEG tidak valid".to_owned());
    }

    let mut document = Document::with_version("1.7");

    let pages_id = document.new_object_id();

    let image_id = document.add_object(image);

    let image_name = b"Im1";

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
            Operation::new("Do", vec![Object::Name(image_name.to_vec())]),
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
        "MediaBox" => vec![
            0.into(),
            0.into(),
            width.into(),
            height.into(),
        ],
        "Contents" => content_id,
    });

    document
        .add_xobject(page_id, image_name, image_id)
        .map_err(|error| format!("Gagal memasang JPEG ke halaman PDF: {error}"))?;

    document.objects.insert(
        pages_id,
        dictionary! {
            "Type" => "Pages",
            "Count" => 1,
            "Kids" => vec![
                page_id.into(),
            ],
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

    Ok(output)
}
