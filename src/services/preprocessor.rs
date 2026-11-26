use image::{imageops::FilterType, GenericImageView, ImageFormat};
use std::io::Cursor;
use crate::error::AppError;

pub struct ImagePreprocessor;

impl ImagePreprocessor {
    /// Preprocess an image for OCR
    pub fn preprocess_for_ocr(image_bytes: &[u8]) -> Result<Vec<u8>, AppError> {
        let img = image::load_from_memory(image_bytes)
            .map_err(|e| AppError::BadRequest(format!("Failed to decode image: {}", e)))?;

        let (orig_width, orig_height) = img.dimensions();

        // Target at least 2200px on the longest dimension for clear OCR text recognition
        let processed_img = if orig_width < 1800 && orig_height < 1800 {
            let max_dim = (orig_width.max(orig_height) as f32).max(1.0);
            let target = 2200.0_f32;
            let scale_factor = (target / max_dim).max(1.5).min(4.0);
            let new_width = (orig_width as f32 * scale_factor) as u32;
            let new_height = (orig_height as f32 * scale_factor) as u32;

            tracing::debug!(
                "Upscaling image from {}x{} to {}x{} for enhanced OCR accuracy",
                orig_width,
                orig_height,
                new_width,
                new_height
            );

            img.resize(new_width, new_height, FilterType::Lanczos3)
        } else {
            img
        };

        // Convert to grayscale for OCR (preserve subtle contrast of faint labels like 'Invoice no.:' and dates)
        let gray = processed_img.grayscale();

        let mut output_bytes = Vec::new();
        gray
            .write_to(&mut Cursor::new(&mut output_bytes), ImageFormat::Png)
            .map_err(|e| AppError::Internal(format!("Failed to encode preprocessed image: {}", e)))?;

        Ok(output_bytes)
    }
}
