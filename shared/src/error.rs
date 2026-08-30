use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ApiError;

    #[test]
    fn serializes_public_error_shape() {
        let error = ApiError::new("invalid_upload", "File tidak valid.");
        let json = serde_json::to_string(&error).expect("ApiError harus dapat diserialisasi");

        assert_eq!(
            json,
            r#"{"code":"invalid_upload","message":"File tidak valid."}"#
        );
    }

    #[test]
    fn round_trips_through_json() {
        let error = ApiError::new("request_timeout", "Request terlalu lama.");
        let json = serde_json::to_string(&error).expect("ApiError harus dapat diserialisasi");
        let restored: ApiError = serde_json::from_str(&json).expect("ApiError harus dapat diparsing");

        assert_eq!(restored, error);
    }
}
