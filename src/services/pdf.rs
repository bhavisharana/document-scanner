use std::fs;
use std::io::Write;
use tokio::process::Command;
use crate::error::AppError;
use crate::services::ocr::OcrEngine;

#[derive(Debug, Clone)]
pub struct PdfExtractResult {
    pub text: String,
    pub confidence: f32,
    pub page_count: usize,
    pub source_type: String,
}

pub struct PdfService;

impl PdfService {
    pub async fn process_pdf(pdf_bytes: &[u8], ocr: &OcrEngine) -> Result<PdfExtractResult, AppError> {
        // Step 1: Try high-fidelity spatial layout extraction via pdftotext -layout
        if let Ok(temp_dir) = tempfile::tempdir() {
            let pdf_path = temp_dir.path().join("document.pdf");
            if let Ok(mut f) = fs::File::create(&pdf_path) {
                if f.write_all(pdf_bytes).is_ok() {
                    let cmd_res = Command::new("pdftotext")
                        .arg("-layout")
                        .arg(&pdf_path)
                        .arg("-")
                        .output()
                        .await;

                    if let Ok(output) = cmd_res {
                        if output.status.success() {
                            let extracted_text = String::from_utf8_lossy(&output.stdout).to_string();
                            if extracted_text.trim().len() > 30 {
                                let quick_check = crate::services::parser::parse_extracted_text(&extracted_text);
                                if !quick_check.items.is_empty() || !quick_check.total_amount.is_empty() || !quick_check.voucher_no.is_empty() {
                                    let page_count = count_pdf_pages(pdf_bytes, &extracted_text);
                                    tracing::info!(
                                        "Layout PDF extracted successfully via pdftotext with {} characters across ~{} page(s)",
                                        extracted_text.trim().len(),
                                        page_count
                                    );
                                    return Ok(PdfExtractResult {
                                        text: extracted_text,
                                        confidence: 100.0,
                                        page_count,
                                        source_type: "digital_pdf".to_string(),
                                    });
                                } else {
                                    tracing::warn!("pdftotext extracted {} chars, but no items/totals/voucher found; falling back to OCR", extracted_text.trim().len());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Step 2: Fallback to native in-memory stream extraction
        match pdf_extract::extract_text_from_mem(pdf_bytes) {
            Ok(extracted_text) if extracted_text.trim().len() > 30 => {
                let quick_check = crate::services::parser::parse_extracted_text(&extracted_text);
                if !quick_check.items.is_empty() || !quick_check.total_amount.is_empty() || !quick_check.voucher_no.is_empty() {
                    let page_count = count_pdf_pages(pdf_bytes, &extracted_text);
                    tracing::info!(
                        "Digital PDF extracted via pdf_extract with {} characters across ~{} page(s)",
                        extracted_text.trim().len(),
                        page_count
                    );
                    return Ok(PdfExtractResult {
                        text: extracted_text,
                        confidence: 100.0,
                        page_count,
                        source_type: "digital_pdf".to_string(),
                    });
                } else {
                    tracing::warn!("pdf_extract found text, but no items/totals/voucher; falling back to OCR");
                }
            }
            Ok(_) => {
                tracing::info!("PDF has little or no embedded text; treating as scanned PDF");
            }
            Err(e) => {
                tracing::warn!("Native PDF extraction failed ({}); falling back to rasterization", e);
            }
        }

        // Step 3: Rasterize scanned PDF into page images using pdftoppm and run OCR
        Self::rasterize_and_ocr(pdf_bytes, ocr).await
    }

    /// Rasterizes PDF to images using pdftoppm and runs OCR on each page
    async fn rasterize_and_ocr(pdf_bytes: &[u8], ocr: &OcrEngine) -> Result<PdfExtractResult, AppError> {
        let temp_dir = tempfile::tempdir()
            .map_err(|e| AppError::Internal(format!("Failed to create temp dir: {}", e)))?;

        let pdf_path = temp_dir.path().join("document.pdf");
        fs::File::create(&pdf_path)
            .and_then(|mut f| f.write_all(pdf_bytes))
            .map_err(|e| AppError::Internal(format!("Failed to write temp PDF: {}", e)))?;

        let output_prefix = temp_dir.path().join("page");

        // Run pdftoppm: -png -r 200 (200 DPI is ideal balance of OCR quality & processing speed)
        let output = Command::new("pdftoppm")
            .arg("-png")
            .arg("-r")
            .arg("200")
            .arg(&pdf_path)
            .arg(&output_prefix)
            .output()
            .await
            .map_err(|e| {
                AppError::PdfError(format!("Failed to execute pdftoppm for scanned PDF: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::PdfError(format!(
                "pdftoppm failed with status {}: {}",
                output.status, stderr
            )));
        }

        // Collect all generated page PNGs
        let mut page_files = Vec::new();
        if let Ok(entries) = fs::read_dir(temp_dir.path()) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "png" {
                        page_files.push(path);
                    }
                }
            }
        }

        // Sort page files naturally by name (page-1.png, page-2.png, ...)
        page_files.sort();

        if page_files.is_empty() {
            return Err(AppError::PdfError("No pages could be rendered from PDF".to_string()));
        }

        let total_pages = page_files.len();
        let mut combined_text = String::new();
        let mut total_confidence = 0.0f32;

        for (idx, page_path) in page_files.iter().enumerate() {
            let page_bytes = fs::read(page_path)
                .map_err(|e| AppError::Internal(format!("Failed to read rendered page: {}", e)))?;

            let (page_text, page_conf) = ocr.scan_image(&page_bytes).await?;
            
            if !combined_text.is_empty() {
                combined_text.push_str("\n\n--- Page Break ---\n\n");
            }
            combined_text.push_str(&page_text);
            total_confidence += page_conf;

            tracing::debug!("Scanned PDF Page {}/{} completed with {}% confidence", idx + 1, total_pages, page_conf);
        }

        let mean_conf = if total_pages > 0 {
            (total_confidence / total_pages as f32).round()
        } else {
            0.0
        };

        Ok(PdfExtractResult {
            text: combined_text,
            confidence: mean_conf,
            page_count: total_pages,
            source_type: "scanned_pdf".to_string(),
        })
    }
}

/// Helper to count PDF pages from lopdf or form feed markers
fn count_pdf_pages(pdf_bytes: &[u8], extracted_text: &str) -> usize {
    // Attempt lopdf page count
    if let Ok(doc) = lopdf::Document::load_mem(pdf_bytes) {
        let pages = doc.get_pages();
        if !pages.is_empty() {
            return pages.len();
        }
    }

    // Fallback: count form feeds (\x0C)
    let form_feeds = extracted_text.matches('\x0C').count();
    if form_feeds > 0 {
        form_feeds
    } else {
        1
    }
}
