use std::str;

const ZIP_LOCAL_FILE_HEADER: [u8; 4] = [0x50, 0x4b, 0x03, 0x04];
const ZIP_CENTRAL_DIRECTORY_HEADER: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];
const ZIP_END_OF_CENTRAL_DIRECTORY: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];

const MAX_PDF_HEADER_SCAN_BYTES: usize = 1024;
const MAX_OFFICE_ZIP_ENTRIES: usize = 2_048;
const MAX_OFFICE_UNCOMPRESSED_BYTES: u64 = 256 * 1024 * 1024;
const MAX_OFFICE_FILENAME_BYTES: usize = 1_024;

pub fn validate_input(bytes: &[u8], format: &str) -> Result<(), String> {
    if bytes.is_empty() {
        return Err(format!("File {format} kosong"));
    }

    match format.to_ascii_lowercase().as_str() {
        "pdf" => validate_pdf_signature(bytes),
        "jpg" | "jpeg" => validate_jpeg_signature(bytes),
        "excel" => validate_office_zip(bytes, "Excel", &["[Content_Types].xml", "xl/workbook.xml"]),
        "word" => validate_office_zip(bytes, "Word", &["[Content_Types].xml", "word/document.xml"]),
        "powerpoint" => validate_office_zip(
            bytes,
            "PowerPoint",
            &["[Content_Types].xml", "ppt/presentation.xml"],
        ),
        _ => Ok(()),
    }
}

fn validate_pdf_signature(bytes: &[u8]) -> Result<(), String> {
    let header_window = bytes.len().min(MAX_PDF_HEADER_SCAN_BYTES);

    if bytes[..header_window]
        .windows(b"%PDF-".len())
        .any(|window| window == b"%PDF-")
    {
        Ok(())
    } else {
        Err("File bukan PDF yang valid".to_owned())
    }
}

fn validate_jpeg_signature(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() >= 3 && bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        Ok(())
    } else {
        Err("File bukan JPEG yang valid".to_owned())
    }
}

fn validate_office_zip(
    bytes: &[u8],
    format: &str,
    required_entries: &[&str],
) -> Result<(), String> {
    if bytes.len() < 22 || !bytes.starts_with(&ZIP_LOCAL_FILE_HEADER) {
        return Err(format!("File {format} bukan Office OOXML yang valid"));
    }

    let search_start = bytes.len().saturating_sub(65_557);
    let eocd_offset = bytes[search_start..]
        .windows(4)
        .rposition(|window| window == ZIP_END_OF_CENTRAL_DIRECTORY)
        .map(|relative| search_start + relative)
        .ok_or_else(|| format!("File {format} bukan ZIP/OOXML yang valid"))?;

    let eocd_end = eocd_offset
        .checked_add(22)
        .ok_or_else(|| format!("Struktur ZIP {format} terlalu besar"))?;
    if eocd_end > bytes.len() {
        return Err(format!("Struktur ZIP {format} terpotong"));
    }

    let disk_number = read_u16(bytes, eocd_offset + 4)?;
    let central_start_disk = read_u16(bytes, eocd_offset + 6)?;
    let entries_on_disk = usize::from(read_u16(bytes, eocd_offset + 8)?);
    let entries_total = usize::from(read_u16(bytes, eocd_offset + 10)?);
    let central_size = u64::from(read_u32(bytes, eocd_offset + 12)?);
    let central_offset = u64::from(read_u32(bytes, eocd_offset + 16)?);
    let comment_length = usize::from(read_u16(bytes, eocd_offset + 20)?);

    if disk_number != 0 || central_start_disk != 0 || entries_on_disk != entries_total {
        return Err(format!(
            "ZIP {format} menggunakan multi-disk yang tidak didukung"
        ));
    }

    if entries_total == usize::from(u16::MAX)
        || central_size == u64::from(u32::MAX)
        || central_offset == u64::from(u32::MAX)
    {
        return Err(format!("ZIP64 untuk {format} tidak didukung"));
    }

    if eocd_end
        .checked_add(comment_length)
        .is_none_or(|end| end != bytes.len())
    {
        return Err(format!("Struktur ZIP {format} tidak konsisten"));
    }

    if entries_total == 0 || entries_total > MAX_OFFICE_ZIP_ENTRIES {
        return Err(format!(
            "Jumlah entry {format} melebihi batas maksimum ({MAX_OFFICE_ZIP_ENTRIES})"
        ));
    }

    let central_start = usize::try_from(central_offset)
        .map_err(|_| format!("Offset central directory {format} terlalu besar"))?;
    let central_length = usize::try_from(central_size)
        .map_err(|_| format!("Ukuran central directory {format} terlalu besar"))?;
    let central_end = central_start
        .checked_add(central_length)
        .ok_or_else(|| format!("Central directory {format} terlalu besar"))?;

    if central_start > eocd_offset || central_end != eocd_offset {
        return Err(format!("Central directory {format} tidak valid"));
    }

    let mut cursor = central_start;
    let mut total_uncompressed = 0_u64;
    let mut seen_required = vec![false; required_entries.len()];
    let mut has_macro_payload = false;

    for _ in 0..entries_total {
        if cursor.checked_add(46).is_none_or(|end| end > central_end)
            || bytes.get(cursor..cursor + 4) != Some(ZIP_CENTRAL_DIRECTORY_HEADER.as_slice())
        {
            return Err(format!("Central directory {format} terpotong atau invalid"));
        }

        let flags = read_u16(bytes, cursor + 8)?;
        let compression = read_u16(bytes, cursor + 10)?;
        let compressed_size = u64::from(read_u32(bytes, cursor + 20)?);
        let uncompressed_size = u64::from(read_u32(bytes, cursor + 24)?);
        let filename_length = usize::from(read_u16(bytes, cursor + 28)?);
        let extra_length = usize::from(read_u16(bytes, cursor + 30)?);
        let comment_length = usize::from(read_u16(bytes, cursor + 32)?);

        if flags & 0x0001 != 0 {
            return Err(format!("{format} terenkripsi tidak didukung"));
        }

        if !matches!(compression, 0 | 8) {
            return Err(format!("Metode kompresi ZIP {format} tidak didukung"));
        }

        total_uncompressed = total_uncompressed
            .checked_add(uncompressed_size)
            .ok_or_else(|| format!("Ukuran hasil ekstraksi {format} terlalu besar"))?;
        if total_uncompressed > MAX_OFFICE_UNCOMPRESSED_BYTES {
            return Err(format!(
                "Ukuran terurai {format} melebihi batas maksimum ({} MiB)",
                MAX_OFFICE_UNCOMPRESSED_BYTES / 1024 / 1024
            ));
        }

        let name_start = cursor + 46;
        let name_end = name_start
            .checked_add(filename_length)
            .ok_or_else(|| format!("Nama file ZIP {format} terlalu panjang"))?;
        let extra_end = name_end
            .checked_add(extra_length)
            .ok_or_else(|| format!("Extra field ZIP {format} terlalu panjang"))?;
        let record_end = extra_end
            .checked_add(comment_length)
            .ok_or_else(|| format!("Entry ZIP {format} terlalu besar"))?;

        if filename_length == 0
            || filename_length > MAX_OFFICE_FILENAME_BYTES
            || record_end > central_end
        {
            return Err(format!("Nama/metadata entry ZIP {format} tidak valid"));
        }

        let name = str::from_utf8(&bytes[name_start..name_end])
            .map_err(|_| format!("Nama entry ZIP {format} bukan UTF-8 yang valid"))?;

        if name.starts_with('/')
            || name.starts_with('\\')
            || name.contains("../")
            || name.contains("..\\")
        {
            return Err(format!("Entry ZIP {format} memiliki path berbahaya"));
        }

        let normalized_name = name.to_ascii_lowercase();
        if normalized_name.ends_with("vbaproject.bin")
            || normalized_name.contains("/macros/")
            || normalized_name.contains("externallinks/")
        {
            has_macro_payload = true;
        }

        for (index, required) in required_entries.iter().enumerate() {
            if name == *required {
                seen_required[index] = true;
            }
        }

        if compressed_size > 0 && uncompressed_size > MAX_OFFICE_UNCOMPRESSED_BYTES {
            return Err(format!("Entry ZIP {format} melebihi batas ukuran terurai"));
        }

        cursor = record_end;
    }

    if cursor != central_end {
        return Err(format!("Central directory {format} memiliki data ekstra"));
    }

    if seen_required.iter().any(|seen| !seen) {
        let missing = required_entries
            .iter()
            .enumerate()
            .filter(|(index, _)| !seen_required[*index])
            .map(|(_, name)| *name)
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "File {format} tidak memiliki entry OOXML wajib: {missing}"
        ));
    }

    if has_macro_payload {
        return Err(format!(
            "File {format} mengandung payload macro/external link yang tidak didukung"
        ));
    }

    Ok(())
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    let slice = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| "Struktur ZIP terpotong".to_owned())?;
    Ok(u16::from_le_bytes([slice[0], slice[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let slice = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| "Struktur ZIP terpotong".to_owned())?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_u16(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn push_u32(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn crc32(data: &[u8]) -> u32 {
        let mut crc = u32::MAX;
        for &byte in data {
            crc ^= u32::from(byte);
            for _ in 0..8 {
                crc = if crc & 1 != 0 {
                    (crc >> 1) ^ 0xedb8_8320
                } else {
                    crc >> 1
                };
            }
        }
        !crc
    }

    fn minimal_ooxml(entries: &[&str]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut central = Vec::new();
        let mut offset = 0_u32;

        for name in entries {
            let name = name.as_bytes();
            let data = b"{}";
            let crc = crc32(data);
            let size = u32::try_from(data.len()).unwrap();

            out.extend_from_slice(&ZIP_LOCAL_FILE_HEADER);
            push_u16(&mut out, 20);
            push_u16(&mut out, 0);
            push_u16(&mut out, 0);
            push_u16(&mut out, 0);
            push_u16(&mut out, 0);
            push_u32(&mut out, crc);
            push_u32(&mut out, size);
            push_u32(&mut out, size);
            push_u16(&mut out, u16::try_from(name.len()).unwrap());
            push_u16(&mut out, 0);
            out.extend_from_slice(name);
            out.extend_from_slice(data);

            central.extend_from_slice(&ZIP_CENTRAL_DIRECTORY_HEADER);
            push_u16(&mut central, 20);
            push_u16(&mut central, 20);
            push_u16(&mut central, 0);
            push_u16(&mut central, 0);
            push_u16(&mut central, 0);
            push_u16(&mut central, 0);
            push_u32(&mut central, crc);
            push_u32(&mut central, size);
            push_u32(&mut central, size);
            push_u16(&mut central, u16::try_from(name.len()).unwrap());
            push_u16(&mut central, 0);
            push_u16(&mut central, 0);
            push_u16(&mut central, 0);
            push_u16(&mut central, 0);
            push_u32(&mut central, 0);
            push_u32(&mut central, offset);
            central.extend_from_slice(name);

            offset = offset
                .checked_add(30 + u32::try_from(name.len()).unwrap() + size)
                .unwrap();
        }

        let central_offset = u32::try_from(out.len()).unwrap();
        let central_size = u32::try_from(central.len()).unwrap();
        out.extend_from_slice(&central);

        out.extend_from_slice(&ZIP_END_OF_CENTRAL_DIRECTORY);
        push_u16(&mut out, 0);
        push_u16(&mut out, 0);
        push_u16(&mut out, u16::try_from(entries.len()).unwrap());
        push_u16(&mut out, u16::try_from(entries.len()).unwrap());
        push_u32(&mut out, central_size);
        push_u32(&mut out, central_offset);
        push_u16(&mut out, 0);

        out
    }

    #[test]
    fn rejects_empty_input() {
        assert!(validate_input(&[], "PDF").is_err());
    }

    #[test]
    fn accepts_pdf_signature_with_leading_bytes() {
        assert!(validate_input(b"%PDF-1.7\n...", "PDF").is_ok());
        assert!(validate_input(b"junk\n%PDF-1.7\n...", "PDF").is_ok());
    }

    #[test]
    fn rejects_non_pdf_with_pdf_format() {
        assert!(validate_input(b"PK\\x03\\x04", "PDF").is_err());
    }

    #[test]
    fn rejects_fake_office_bytes() {
        assert!(validate_input(b"anything", "Excel").is_err());
        assert!(validate_input(b"PK\\x03\\x04not-ooxml", "Word").is_err());
    }

    #[test]
    fn accepts_minimal_valid_office_containers() {
        let excel = minimal_ooxml(&["[Content_Types].xml", "xl/workbook.xml"]);
        let word = minimal_ooxml(&["[Content_Types].xml", "word/document.xml"]);
        let powerpoint = minimal_ooxml(&["[Content_Types].xml", "ppt/presentation.xml"]);

        assert!(validate_input(&excel, "Excel").is_ok());
        assert!(validate_input(&word, "Word").is_ok());
        assert!(validate_input(&powerpoint, "PowerPoint").is_ok());
    }

    #[test]
    fn rejects_macro_or_external_link_markers() {
        let macro_doc = minimal_ooxml(&[
            "[Content_Types].xml",
            "word/document.xml",
            "word/vbaProject.bin",
        ]);
        let external_link = minimal_ooxml(&[
            "[Content_Types].xml",
            "xl/workbook.xml",
            "xl/externalLinks/externalLink1.xml",
        ]);

        assert!(validate_input(&macro_doc, "Word").is_err());
        assert!(validate_input(&external_link, "Excel").is_err());
    }

    #[test]
    fn rejects_zip_path_traversal() {
        let malicious = minimal_ooxml(&["[Content_Types].xml", "../xl/workbook.xml"]);
        assert!(validate_input(&malicious, "Excel").is_err());
    }

    #[test]
    fn rejects_non_jpeg() {
        assert!(validate_input(b"GIF89a", "JPG").is_err());
    }

    #[test]
    fn non_supported_formats_remain_non_empty_only() {
        assert!(validate_input(b"anything", "unknown").is_ok());
    }
}
