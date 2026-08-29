use lopdf::{Document, Object};

fn main() {
    let path = "/home/sync/pdfin-test/result-excel.pdf";

    let doc = Document::load(path).expect("Gagal membuka PDF");

    println!("Pages: {}", doc.get_pages().len());

    for (object_id, object) in &doc.objects {
        let Object::Stream(stream) = object else {
            continue;
        };

        if stream.dict.get(b"Subtype").ok() != Some(&Object::Name(b"XML".to_vec())) {
            continue;
        }

        println!("Metadata object: {:?}", object_id);
    }

    let pages = doc.get_pages();

    for (page_number, page_id) in pages {
        println!("\n================ PAGE {} ================", page_number);

        let content_bytes = doc.get_page_content(page_id);

        let content = match lopdf::content::Content::decode(&content_bytes) {
            Ok(content) => content,
            Err(error) => {
                println!("Decode content gagal: {error}");
                continue;
            }
        };

        for operation in content.operations {
            match operation.operator.as_str() {
                "Tj" | "TJ" => {
                    println!("{} {:?}", operation.operator, operation.operands);
                }

                _ => {}
            }
        }
    }

    println!("\n================ FONTS ================");

    for (object_id, object) in &doc.objects {
        let Object::Dictionary(dict) = object else {
            continue;
        };

        if dict.get(b"Type").ok() != Some(&Object::Name(b"Font".to_vec())) {
            continue;
        }

        println!("\nFONT {:?}", object_id);
        println!("{:#?}", dict);

        if let Ok(Object::Reference(to_unicode_id)) = dict.get(b"ToUnicode") {
            println!("ToUnicode: {:?}", to_unicode_id);

            if let Some(Object::Stream(stream)) = doc.objects.get(to_unicode_id) {
                match stream.decompressed_content() {
                    Ok(data) => {
                        println!("{}", String::from_utf8_lossy(&data));
                    }

                    Err(error) => {
                        println!("Gagal decompress ToUnicode: {error}");
                    }
                }
            }
        }
    }
}
