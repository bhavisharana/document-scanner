use axum::{
    extract::{Multipart, State},
    response::Json,
};
use std::sync::Arc;
use std::time::Instant;

use crate::config::Config;
use crate::error::AppError;
use crate::models::document::ScanResponse;
use crate::services::{ocr::OcrEngine, parser, pdf::PdfService};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub ocr: Arc<OcrEngine>,
}

/// Endpoint: POST /api/documents/scan
/// Accepts multipart/form-data with file upload
pub async fn scan_document(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<ScanResponse>, AppError> {
    let start_time = Instant::now();
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name = String::new();
    let mut content_type = String::new();

    while let Some(field) = multipart.next_field().await? {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "file" || field.file_name().is_some() {
            file_name = field.file_name().unwrap_or("document").to_string();
            content_type = field.content_type().unwrap_or("").to_string();

            let data = field.bytes().await?;
            if data.len() > state.config.max_file_size_bytes {
                return Err(AppError::PayloadTooLarge(format!(
                    "File size ({} bytes) exceeds the allowed limit of {} MB",
                    data.len(),
                    state.config.max_file_size_bytes / (1024 * 1024)
                )));
            }

            file_bytes = Some(data.to_vec());
            break;
        }
    }

    let bytes = file_bytes.ok_or_else(|| {
        AppError::BadRequest("No file uploaded. Please upload a file with the form field name 'file'".to_string())
    })?;

    if bytes.is_empty() {
        return Err(AppError::BadRequest("Uploaded file is empty".to_string()));
    }

    let lower_name = file_name.to_lowercase();
    let is_pdf = lower_name.ends_with(".pdf")
        || content_type == "application/pdf"
        || bytes.starts_with(b"%PDF-");

    let is_image = lower_name.ends_with(".png")
        || lower_name.ends_with(".jpg")
        || lower_name.ends_with(".jpeg")
        || lower_name.ends_with(".webp")
        || lower_name.ends_with(".tiff")
        || lower_name.ends_with(".bmp")
        || content_type.starts_with("image/");

    if !is_pdf && !is_image {
        return Err(AppError::UnsupportedFileType(format!(
            "Unsupported file format for '{}'. Only PDF, PNG, JPG, JPEG, WEBP, and TIFF are supported.",
            file_name
        )));
    }

    tracing::info!(
        "Received file '{}' ({} bytes, detected_as: {}) for processing",
        file_name,
        bytes.len(),
        if is_pdf { "PDF" } else { "Image" }
    );

    // Hybrid processing pipeline
    let (raw_text, confidence, page_count, source_type) = if is_pdf {
        let result = PdfService::process_pdf(&bytes, &state.ocr).await?;
        (
            result.text,
            result.confidence,
            result.page_count,
            result.source_type,
        )
    } else {
        let (text, conf) = state.ocr.scan_image(&bytes).await?;
        (text, conf, 1, "image".to_string())
    };

    // Extract structured business fields from extracted text
    let fields = parser::parse_extracted_text(&raw_text);

    let is_valid = fields.is_valid_document;
    let message = fields.validation_message.clone();

    let processing_time_ms = start_time.elapsed().as_millis();

    tracing::info!(
        "Processed '{}' in {}ms (confidence: {}%, is_valid: {}, doc_type: '{}', voucher: '{}')",
        file_name,
        processing_time_ms,
        confidence,
        is_valid,
        fields.document_type,
        fields.voucher_no
    );

    Ok(Json(ScanResponse {
        success: true,
        is_valid_document: is_valid,
        message,
        fields,
        raw_text,
        confidence,
        processing_time_ms,
        page_count,
        source_type,
    }))
}

/// Endpoint: GET /api/health
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "rust-document-scanner",
        "version": env!("CARGO_PKG_VERSION"),
        "tesseract_path": state.config.tesseract_path,
        "max_file_size_mb": state.config.max_file_size_bytes / (1024 * 1024),
    }))
}

/// Endpoint: GET /
pub async fn root() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "Rust Document Scanner API",
        "version": env!("CARGO_PKG_VERSION"),
        "endpoints": {
            "POST /api/documents/scan": "Upload a PDF or Image (form-data: 'file') to scan and extract structured data",
            "GET /api/health": "Health and service status check",
        }
    }))
}
