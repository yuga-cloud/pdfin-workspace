use std::io::{self, Write};

const ZIP_LOCAL_FILE_HEADER: u32 = 0x0403_4b50;
const ZIP_CENTRAL_DIRECTORY_HEADER: u32 = 0x0201_4b50;
const ZIP_END_OF_CENTRAL_DIRECTORY: u32 = 0x0605_4b50;
const ZIP_VERSION_NEEDED: u16 = 20;
const MAX_ZIP_ENTRIES: usize = 100;
const MAX_ZIP_SIZE: usize = 512 * 1024 * 1024;

pub struct ZipEntry<'a> {
    pub name: &'a str,
    pub data: &'a [u8],
}

pub fn create_stored_zip(entries: &[ZipEntry<'_>]) -> Result<Vec<u8>, String> {
    if entries.is_empty() {
        return Err("ZIP archive tidak memiliki file".to_owned());
    }

    if entries.len() > MAX_ZIP_ENTRIES {
        return Err(format!("Jumlah file ZIP melebihi batas maksimum ({MAX_ZIP_ENTRIES})"));
    }

    let estimated_size = entries.iter().try_fold(0usize, |total, entry| {
        let name_len = entry.name.len();
        if name_len > u16::MAX as usize {
            return Err("Nama file ZIP terlalu panjang".to_owned());
        }
        if entry.data.len() > u32::MAX as usize {
            return Err("File ZIP terlalu besar".to_owned());
        }
        total
            .checked_add(30)
            .and_then(|value| value.checked_add(name_len))
            .and_then(|value| value.checked_add(entry.data.len()))
            .ok_or_else(|| "Ukuran ZIP melebihi kapasitas yang didukung".to_owned())
    })?;

    if estimated_size > MAX_ZIP_SIZE {
        return Err("Ukuran ZIP melebihi batas maksimum".to_owned());
    }

    let mut output = Vec::with_capacity(estimated_size);
    let mut central_directory = Vec::new();
    let mut offset = 0u32;

    for entry in entries {
        let name = entry.name.as_bytes();
        let crc = crc32(entry.data);
        let size = u32::try_from(entry.data.len()).map_err(|_| "File ZIP terlalu besar".to_owned())?;
        let name_len = u16::try_from(name.len()).map_err(|_| "Nama file ZIP terlalu panjang".to_owned())?;

        write_u32(&mut output, ZIP_LOCAL_FILE_HEADER)?;
        write_u16(&mut output, ZIP_VERSION_NEEDED)?;
        write_u16(&mut output, 0)?;
        write_u16(&mut output, 0)?;
        write_u16(&mut output, 0)?;
        write_u16(&mut output, 0)?;
        write_u32(&mut output, crc)?;
        write_u32(&mut output, size)?;
        write_u32(&mut output, size)?;
        write_u16(&mut output, name_len)?;
        write_u16(&mut output, 0)?;
        output.write_all(name).map_err(|error| error.to_string())?;
        output.write_all(entry.data).map_err(|error| error.to_string())?;

        write_u32(&mut central_directory, ZIP_CENTRAL_DIRECTORY_HEADER)?;
        write_u16(&mut central_directory, ZIP_VERSION_NEEDED)?;
        write_u16(&mut central_directory, ZIP_VERSION_NEEDED)?;
        write_u16(&mut central_directory, 0)?;
        write_u16(&mut central_directory, 0)?;
        write_u16(&mut central_directory, 0)?;
        write_u16(&mut central_directory, 0)?;
        write_u32(&mut central_directory, crc)?;
        write_u32(&mut central_directory, size)?;
        write_u32(&mut central_directory, size)?;
        write_u16(&mut central_directory, name_len)?;
        write_u16(&mut central_directory, 0)?;
        write_u16(&mut central_directory, 0)?;
        write_u16(&mut central_directory, 0)?;
        write_u16(&mut central_directory, 0)?;
        write_u32(&mut central_directory, 0)?;
        write_u32(&mut central_directory, offset)?;
        central_directory
            .write_all(name)
            .map_err(|error| error.to_string())?;

        offset = offset
            .checked_add(30)
            .and_then(|value| value.checked_add(u32::from(name_len)))
            .and_then(|value| value.checked_add(size))
            .ok_or_else(|| "Ukuran ZIP melebihi batas ZIP32".to_owned())?;
    }

    let central_offset = u32::try_from(output.len()).map_err(|_| "Ukuran ZIP melebihi batas ZIP32".to_owned())?;
    let central_size = u32::try_from(central_directory.len()).map_err(|_| "Ukuran ZIP melebihi batas ZIP32".to_owned())?;

    output.extend_from_slice(&central_directory);
    write_u32(&mut output, ZIP_END_OF_CENTRAL_DIRECTORY)?;
    write_u16(&mut output, 0)?;
    write_u16(&mut output, 0)?;
    let entry_count = u16::try_from(entries.len()).map_err(|_| "Terlalu banyak entry ZIP".to_owned())?;
    write_u16(&mut output, entry_count)?;
    write_u16(&mut output, entry_count)?;
    write_u32(&mut output, central_size)?;
    write_u32(&mut output, central_offset)?;
    write_u16(&mut output, 0)?;

    if output.len() > MAX_ZIP_SIZE {
        return Err("Ukuran ZIP melebihi batas maksimum".to_owned());
    }

    Ok(output)
}

fn write_u16(output: &mut Vec<u8>, value: u16) -> io::Result<()> {
    output.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn write_u32(output: &mut Vec<u8>, value: u32) -> io::Result<()> {
    output.extend_from_slice(&value.to_le_bytes());
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_valid_stored_zip_shape() {
        let entries = [
            ZipEntry {
                name: "one.txt",
                data: b"hello",
            },
            ZipEntry {
                name: "two.txt",
                data: b"world",
            },
        ];

        let archive = create_stored_zip(&entries).expect("ZIP harus terbentuk");
        assert_eq!(&archive[0..4], &ZIP_LOCAL_FILE_HEADER.to_le_bytes());
        assert!(archive.windows(4).any(|window| window == ZIP_CENTRAL_DIRECTORY_HEADER.to_le_bytes()));
        assert!(archive.windows(4).any(|window| window == ZIP_END_OF_CENTRAL_DIRECTORY.to_le_bytes()));
    }

    #[test]
    fn rejects_empty_archive() {
        assert!(create_stored_zip(&[]).is_err());
    }
}
