use std::collections::HashMap;

use lopdf::{Document, Object};

type UnicodeMap = HashMap<u8, char>;

fn main() {
    let path = "/home/sync/pdfin-test/result-excel.pdf";

    let doc = Document::load(path).expect("Gagal membuka PDF");

    let pages = doc.get_pages();

    for (page_number, page_id) in pages {
        println!();
        println!("================ PAGE {} ================", page_number);

        let fonts = collect_page_fonts(&doc, page_id);

        let content_bytes = doc.get_page_content(page_id);

        let content =
            lopdf::content::Content::decode(&content_bytes).expect("Gagal decode content stream");

        let mut current_x = 0.0_f64;
        let mut current_y = 0.0_f64;

        let mut current_font: Option<String> = None;

        for operation in content.operations {
            match operation.operator.as_str() {
                "Tf" => {
                    if let Some(Object::Name(font_name)) = operation.operands.first() {
                        let name = String::from_utf8_lossy(font_name).to_string();

                        current_font = Some(name);

                        println!("\nFONT = {}", current_font.as_deref().unwrap());
                    }
                }

                "Td" | "TD" => {
                    if operation.operands.len() >= 2 {
                        current_x = number(&operation.operands[0]);
                        current_y = number(&operation.operands[1]);
                    }
                }

                "Tm" => {
                    if operation.operands.len() >= 6 {
                        current_x = number(&operation.operands[4]);
                        current_y = number(&operation.operands[5]);
                    }
                }

                "Tj" => {
                    if let Some(Object::String(bytes, _)) = operation.operands.first() {
                        let text = decode_pdf_string(bytes, current_font.as_deref(), &fonts);

                        println!("TEXT x={:.3} y={:.3} {:?}", current_x, current_y, text);
                    }
                }

                "TJ" => {
                    let Some(Object::Array(items)) = operation.operands.first() else {
                        continue;
                    };

                    let mut text = String::new();

                    for item in items {
                        if let Object::String(bytes, _) = item {
                            text.push_str(&decode_pdf_string(
                                bytes,
                                current_font.as_deref(),
                                &fonts,
                            ));
                        }
                    }

                    if !text.is_empty() {
                        println!("TEXT x={:.3} y={:.3} {:?}", current_x, current_y, text);
                    }
                }

                _ => {}
            }
        }
    }
}

fn number(object: &Object) -> f64 {
    match object {
        Object::Integer(value) => *value as f64,
        Object::Real(value) => *value as f64,
        _ => 0.0,
    }
}

fn collect_page_fonts(doc: &Document, page_id: lopdf::ObjectId) -> HashMap<String, UnicodeMap> {
    let mut result = HashMap::new();

    let Ok(Object::Dictionary(page)) = doc.get_object(page_id) else {
        return result;
    };

    let Ok(resources) = page.get(b"Resources") else {
        return result;
    };

    let resources_id = match resources {
        Object::Reference(id) => *id,
        _ => return result,
    };

    let Ok(Object::Dictionary(resources)) = doc.get_object(resources_id) else {
        return result;
    };

    let Ok(fonts) = resources.get(b"Font") else {
        return result;
    };

    let font_id = match fonts {
        Object::Reference(id) => *id,
        _ => return result,
    };

    let Ok(Object::Dictionary(fonts)) = doc.get_object(font_id) else {
        return result;
    };

    for (font_name, font_object) in fonts.iter() {
        let font_id = match font_object {
            Object::Reference(id) => *id,
            _ => continue,
        };

        let Ok(Object::Dictionary(font)) = doc.get_object(font_id) else {
            continue;
        };

        let Ok(to_unicode) = font.get(b"ToUnicode") else {
            continue;
        };

        let to_unicode_id = match to_unicode {
            Object::Reference(id) => *id,
            _ => continue,
        };

        let Some(Object::Stream(stream)) = doc.objects.get(&to_unicode_id) else {
            continue;
        };

        let Ok(data) = stream.decompressed_content() else {
            continue;
        };

        let map = parse_to_unicode(&data);

        let name = String::from_utf8_lossy(font_name).to_string();

        result.insert(name, map);
    }

    result
}

fn parse_to_unicode(data: &[u8]) -> UnicodeMap {
    let text = String::from_utf8_lossy(data);

    let mut map = HashMap::new();

    for line in text.lines() {
        let line = line.trim();

        if !line.starts_with('<') {
            continue;
        }

        let Some(end_source) = line.find('>') else {
            continue;
        };

        let source_hex = &line[1..end_source];

        let rest = &line[end_source + 1..];

        let Some(start_target) = rest.find('<') else {
            continue;
        };

        let rest = &rest[start_target + 1..];

        let Some(end_target) = rest.find('>') else {
            continue;
        };

        let target_hex = &rest[..end_target];

        if source_hex.len() != 2 {
            continue;
        }

        let Ok(source) = u8::from_str_radix(source_hex, 16) else {
            continue;
        };

        let Ok(target) = u32::from_str_radix(target_hex, 16) else {
            continue;
        };

        let Some(character) = char::from_u32(target) else {
            continue;
        };

        map.insert(source, character);
    }

    map
}

fn decode_pdf_string(
    bytes: &[u8],
    font_name: Option<&str>,
    fonts: &HashMap<String, UnicodeMap>,
) -> String {
    let Some(font_name) = font_name else {
        return String::from_utf8_lossy(bytes).to_string();
    };

    let Some(map) = fonts.get(font_name) else {
        return String::from_utf8_lossy(bytes).to_string();
    };

    bytes
        .iter()
        .map(|byte| map.get(byte).copied().unwrap_or('\u{FFFD}'))
        .collect()
}
