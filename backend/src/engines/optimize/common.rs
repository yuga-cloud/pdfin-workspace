pub fn validate_input(bytes: &[u8], format: &str) -> Result<(), String> {
    if bytes.is_empty() {
        return Err(format!("File {format} kosong"));
    }

    Ok(())
}
