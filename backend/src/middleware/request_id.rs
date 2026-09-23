//! Middleware request ID.
//!
//! Setiap request mendapat `x-request-id` (di-echo dari header klien bila ada,
//! atau dibuatkan otomatis), disimpan di request extensions untuk keperluan
//! logging, dan selalu dikembalikan di response. Untuk response error JSON,
//! request ID juga disisipkan ke body `{"error": {..., "request_id": ...}}`
//! sehingga error contract selalu bisa dikorelasikan dengan log server.

use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::SystemTime,
};

use axum::{
    body::{Body, to_bytes},
    extract::Request,
    http::{HeaderValue, header},
    middleware::Next,
    response::Response,
};

pub const X_REQUEST_ID_HEADER: &str = "x-request-id";
pub const X_REQUEST_ID_HEADER_NAME: header::HeaderName =
    header::HeaderName::from_static(X_REQUEST_ID_HEADER);
const MAX_INCOMING_REQUEST_ID_LENGTH: usize = 128;
const MAX_JSON_ERROR_BODY_BYTES: usize = 1024 * 1024;

static REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub struct RequestId(pub String);

/// Menghasilkan request ID baru (timestamp milidetik + counter) tanpa
/// dependensi eksternal; cukup unik untuk korelasi dalam satu proses server.
fn generate_request_id() -> String {
    let millis = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    let sequence = REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("req-{millis}-{sequence:06x}")
}

/// Mengambil request ID yang dikirim klien via header `x-request-id`, jika
/// ada dan masuk akal (tidak kosong, panjang wajar).
fn incoming_request_id(request: &Request) -> Option<String> {
    let value = request.headers().get(X_REQUEST_ID_HEADER_NAME)?;
    let text = value.to_str().ok()?.trim();
    if text.is_empty() || text.len() > MAX_INCOMING_REQUEST_ID_LENGTH {
        return None;
    }
    Some(text.to_owned())
}

/// Menentukan request ID untuk sebuah request.
pub fn request_id_for(request: &Request) -> String {
    incoming_request_id(request).unwrap_or_else(generate_request_id)
}

pub async fn attach_request_id(mut request: Request, next: Next) -> Response {
    let request_id = request_id_for(&request);
    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    let mut response = next.run(request).await;

    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response
            .headers_mut()
            .insert(X_REQUEST_ID_HEADER_NAME, value);
    }

    if response.status().is_client_error() || response.status().is_server_error() {
        response = embed_request_id_in_error_body(response, &request_id).await;
    }

    response
}

/// Menyisipkan `request_id` ke body JSON error `{"error": {...}}` bila respons
/// adalah error dengan content-type JSON.
async fn embed_request_id_in_error_body(response: Response, request_id: &str) -> Response {
    let is_json = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|content_type| content_type.contains("application/json"))
        .unwrap_or(false);

    if !is_json {
        return response;
    }

    let (mut parts, body) = response.into_parts();

    let bytes = match to_bytes(body, MAX_JSON_ERROR_BODY_BYTES).await {
        Ok(bytes) => bytes,
        Err(_) => {
            parts.headers.remove(header::CONTENT_LENGTH);
            return Response::from_parts(parts, Body::empty());
        }
    };

    let updated = match apply_request_id_to_json(&bytes, request_id) {
        Some(updated) => updated,
        None => {
            parts.headers.remove(header::CONTENT_LENGTH);
            return Response::from_parts(parts, Body::from(bytes));
        }
    };

    parts.headers.remove(header::CONTENT_LENGTH);
    Response::from_parts(parts, Body::from(updated))
}

/// Injeksi request ID ke objek `error` pada payload JSON `{"error": {...}}`.
/// Tidak mengubah apa pun bila bentuk payload tidak sesuai.
fn apply_request_id_to_json(body: &[u8], request_id: &str) -> Option<Vec<u8>> {
    let mut value: serde_json::Value = serde_json::from_slice(body).ok()?;
    let error = value.get_mut("error")?;
    let object = error.as_object_mut()?;
    object.insert(
        "request_id".to_owned(),
        serde_json::Value::String(request_id.to_owned()),
    );
    serde_json::to_vec(&value).ok()
}

#[cfg(test)]
mod tests {
    use super::{apply_request_id_to_json, generate_request_id};

    #[test]
    fn generates_unique_and_consistent_request_ids() {
        let first = generate_request_id();
        let second = generate_request_id();
        assert!(first.starts_with("req-"));
        assert_ne!(first, second);
    }

    #[test]
    fn injects_request_id_into_error_envelope() {
        let body = br#"{"error":{"code":"invalid_input","message":"PDF tidak valid."}}"#;
        let updated =
            apply_request_id_to_json(body, "req-123-000001").expect("body harus di-update");
        let value: serde_json::Value =
            serde_json::from_slice(&updated).expect("JSON harus tetap valid");
        assert_eq!(value["error"]["request_id"], "req-123-000001");
        assert_eq!(value["error"]["code"], "invalid_input");
        assert_eq!(value["error"]["message"], "PDF tidak valid.");
    }

    #[test]
    fn leaves_non_error_shape_untouched() {
        let body = br#"{"ok":true}"#;
        assert!(apply_request_id_to_json(body, "req-1").is_none());
    }
}
