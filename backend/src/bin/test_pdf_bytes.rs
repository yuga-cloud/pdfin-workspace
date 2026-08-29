use lopdf::{Document, Object, content::Content};

fn main() {
    let path = "/home/sync/pdfin-test/result-excel.pdf";

    let doc = Document::load(path).expect("Gagal membuka PDF");

    for (page_number, page_id) in doc.get_pages() {
        println!("\n================ PAGE {page_number} ================");

        let bytes = doc.get_page_content(page_id);

        let content = Content::decode(&bytes).expect("Gagal decode content");

        for operation in content.operations {
            match operation.operator.as_str() {
                "Tj" | "TJ" => {
                    println!("\nOPERATOR: {}", operation.operator);

                    if let Some(first) = operation.operands.first() {
                        print_object(first, 0);
                    }
                }

                _ => {}
            }
        }
    }
}

fn print_object(object: &Object, depth: usize) {
    let indent = " ".repeat(depth * 2);

    match object {
        Object::String(bytes, format) => {
            println!("{indent}STRING");
            println!("{indent}  format = {format:?}");
            println!("{indent}  len    = {}", bytes.len());
            println!("{indent}  hex    = {}", hex(bytes));
            println!("{indent}  bytes  = {bytes:?}");
        }

        Object::Array(items) => {
            println!("{indent}ARRAY len={}", items.len());

            for item in items {
                print_object(item, depth + 1);
            }
        }

        other => {
            println!("{indent}{other:?}");
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}
