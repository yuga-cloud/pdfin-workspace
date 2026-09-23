use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    /// ID request yang dikorelasikan dengan log server. Diisi middleware
    /// `x-request-id`; `None` bila ID belum tersedia.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            request_id: None,
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
        let restored: ApiError =
            serde_json::from_str(&json).expect("ApiError harus dapat diparsing");

        assert_eq!(restored, error);
    }

    #[test]
    fn serializes_request_id_when_present() {
        let mut error = ApiError::new("invalid_input", "PDF tidak valid.");
        error.request_id = Some("req-1-000001".to_owned());
        let json = serde_json::to_string(&error).expect("ApiError harus dapat diserialisasi");

        assert_eq!(
            json,
            r#"{"code":"invalid_input","message":"PDF tidak valid.","request_id":"req-1-000001"}"#
        );
    }
}
