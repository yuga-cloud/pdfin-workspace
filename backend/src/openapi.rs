use utoipa::OpenApi;

/// OpenAPI document for the public HTTP API.
///
/// Paths and schemas are added incrementally as API contracts stabilize.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "PDFin API",
        version = "1.0.0",
        description = "High performance PDF processing API"
    ),
    tags(
        (name = "health", description = "Service health endpoints"),
        (name = "files", description = "File management endpoints"),
        (name = "conversions", description = "PDF conversion jobs"),
        (name = "jobs", description = "Background job tracking")
    )
)]
pub struct ApiDoc;
