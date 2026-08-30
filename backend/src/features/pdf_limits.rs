pub const MAX_MERGE_FILES: usize = 50;
pub const MAX_SPLIT_RANGES: usize = 100;
pub const MAX_SPLIT_RANGES_LENGTH: usize = 4 * 1024;
pub const MAX_PAGE_ORDER_LENGTH: usize = 16 * 1024;

pub fn validate_merge_file_count(count: usize) -> Result<(), &'static str> {
    if count > MAX_MERGE_FILES {
        Err("too_many_files")
    } else {
        Ok(())
    }
}

pub fn validate_ranges_length(length: usize) -> Result<(), &'static str> {
    if length > MAX_SPLIT_RANGES_LENGTH {
        Err("ranges_too_long")
    } else {
        Ok(())
    }
}

pub fn validate_range_count(count: usize) -> Result<(), &'static str> {
    if count > MAX_SPLIT_RANGES {
        Err("too_many_ranges")
    } else {
        Ok(())
    }
}

pub fn validate_page_order_length(length: usize) -> Result<(), &'static str> {
    if length > MAX_PAGE_ORDER_LENGTH {
        Err("pages_too_long")
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_file_limit_is_enforced() {
        assert!(validate_merge_file_count(MAX_MERGE_FILES).is_ok());
        assert!(validate_merge_file_count(MAX_MERGE_FILES + 1).is_err());
    }

    #[test]
    fn split_limits_are_enforced() {
        assert!(validate_ranges_length(MAX_SPLIT_RANGES_LENGTH).is_ok());
        assert!(validate_ranges_length(MAX_SPLIT_RANGES_LENGTH + 1).is_err());
        assert!(validate_range_count(MAX_SPLIT_RANGES).is_ok());
        assert!(validate_range_count(MAX_SPLIT_RANGES + 1).is_err());
    }

    #[test]
    fn page_order_limit_is_enforced() {
        assert!(validate_page_order_length(MAX_PAGE_ORDER_LENGTH).is_ok());
        assert!(validate_page_order_length(MAX_PAGE_ORDER_LENGTH + 1).is_err());
    }
}
