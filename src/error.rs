use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    PayloadTooLarge(String),
    UnsupportedFileType(String),
    OcrError(String),
    PdfError(String),
    Internal(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            AppError::PayloadTooLarge(msg) => write!(f, "Payload Too Large: {}", msg),
            AppError::UnsupportedFileType(msg) => write!(f, "Unsupported File Type: {}", msg),
            AppError::OcrError(msg) => write!(f, "OCR Error: {}", msg),
            AppError::PdfError(msg) => write!(f, "PDF Error: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal Server Error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::BadRequest(ref msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::PayloadTooLarge(ref msg) => (StatusCode::PAYLOAD_TOO_LARGE, msg.clone()),
            AppError::UnsupportedFileType(ref msg) => (StatusCode::UNSUPPORTED_MEDIA_TYPE, msg.clone()),
            AppError::OcrError(ref msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("OCR processing failed: {}", msg)),
            AppError::PdfError(ref msg) => (StatusCode::UNPROCESSABLE_ENTITY, format!("PDF extraction failed: {}", msg)),
            AppError::Internal(ref msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        tracing::error!("Request error [{}]: {}", status, error_message);

        let body = Json(json!({
            "success": false,
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<axum::extract::multipart::MultipartError> for AppError {
    fn from(err: axum::extract::multipart::MultipartError) -> Self {
        AppError::BadRequest(err.to_string())
    }
}
