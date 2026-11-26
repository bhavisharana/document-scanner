use std::io::Write;
use std::path::PathBuf;
use tempfile::Builder;
use tokio::process::Command;
use crate::error::AppError;
use crate::services::preprocessor::ImagePreprocessor;

#[derive(Clone, Debug)]
pub struct OcrEngine {
    tesseract_path: String,
    tessdata_prefix: Option<PathBuf>,
}

impl OcrEngine {
    pub fn new(tesseract_path: String, tessdata_prefix: Option<String>) -> Self {
        let prefix = tessdata_prefix.map(PathBuf::from);
        Self {
            tesseract_path,
            tessdata_prefix: prefix,
        }
    }

    /// Perform OCR on image bytes, returning the extracted text and mean confidence score (0.0 - 100.0)
    pub async fn scan_image(&self, raw_image_bytes: &[u8]) -> Result<(String, f32), AppError> {
        // Step 1: Preprocess image for optimal OCR resolution and contrast
        let optimized_bytes = match ImagePreprocessor::preprocess_for_ocr(raw_image_bytes) {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::warn!("Image preprocessing failed ({}); attempting OCR on raw image", e);
                raw_image_bytes.to_vec()
            }
        };

        // Step 2: Write to a temporary PNG file for Tesseract CLI
        let mut temp_file = Builder::new()
            .suffix(".png")
            .tempfile()
            .map_err(|e| AppError::Internal(format!("Failed to create temporary file: {}", e)))?;

        temp_file
            .write_all(&optimized_bytes)
            .map_err(|e| AppError::Internal(format!("Failed to write image to temp file: {}", e)))?;

        let temp_path = temp_file.path().to_path_buf();

        // Step 3: Run Tesseract with TSV output to get words and per-word confidence
        let mut cmd = Command::new(&self.tesseract_path);
        cmd.arg(&temp_path)
            .arg("stdout")
            .arg("--dpi")
            .arg("300")
            .arg("-c")
            .arg("preserve_interword_spaces=1")
            .arg("-l")
            .arg("eng")
            .arg("tsv");

        if let Some(ref prefix) = self.tessdata_prefix {
            cmd.env("TESSDATA_PREFIX", prefix);
        }

        let output = cmd.output().await.map_err(|e| {
            AppError::OcrError(format!("Failed to execute Tesseract at {}: {}", self.tesseract_path, e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Tesseract TSV failed with status {}: {}", output.status, stderr);
            // Fallback to basic text output
            return self.scan_image_plain_text(&temp_path).await;
        }

        let tsv_content = String::from_utf8_lossy(&output.stdout);
        let (extracted_text, confidence) = parse_tsv(&tsv_content);

        // If TSV produced empty text, fallback to plain text execution
        if extracted_text.trim().is_empty() {
            return self.scan_image_plain_text(&temp_path).await;
        }

        Ok((extracted_text, confidence))
    }

    /// Fallback plain text OCR execution
    async fn scan_image_plain_text(&self, temp_path: &std::path::Path) -> Result<(String, f32), AppError> {
        let mut cmd = Command::new(&self.tesseract_path);
        cmd.arg(temp_path)
            .arg("stdout")
            .arg("--dpi")
            .arg("300")
            .arg("-c")
            .arg("preserve_interword_spaces=1")
            .arg("-l")
            .arg("eng");

        if let Some(ref prefix) = self.tessdata_prefix {
            cmd.env("TESSDATA_PREFIX", prefix);
        }

        let output = cmd.output().await.map_err(|e| {
            AppError::OcrError(format!("Failed to execute fallback Tesseract: {}", e))
        })?;

        let text = String::from_utf8_lossy(&output.stdout).to_string();
        let confidence = if text.trim().is_empty() { 0.0 } else { 75.0 };

        Ok((text, confidence))
    }
}

/// Parse Tesseract TSV output into formatted lines and calculate mean confidence score
fn parse_tsv(tsv: &str) -> (String, f32) {
    let mut total_confidence = 0.0f32;
    let mut word_count = 0usize;

    let mut full_text = String::new();
    let mut current_line = String::new();
    let mut last_line_id = (-1i32, -1i32, -1i32); // (block_num, par_num, line_num)

    for line in tsv.lines().skip(1) {
        let cols: Vec<&str> = line.split('\t').collect();
        // TSV format: level, page_num, block_num, par_num, line_num, word_num, left, top, width, height, conf, text
        if cols.len() < 12 {
            continue;
        }

        let level: i32 = cols[0].parse().unwrap_or(-1);
        if level != 5 {
            continue; // Level 5 is word level
        }

        let block_num: i32 = cols[2].parse().unwrap_or(-1);
        let par_num: i32 = cols[3].parse().unwrap_or(-1);
        let line_num: i32 = cols[4].parse().unwrap_or(-1);
        let conf: f32 = cols[10].parse().unwrap_or(-1.0);
        let word = cols[11].trim();

        if word.is_empty() {
            continue;
        }

        if conf >= 0.0 {
            total_confidence += conf;
            word_count += 1;
        }

        let this_line_id = (block_num, par_num, line_num);
        if this_line_id != last_line_id {
            if !current_line.is_empty() {
                full_text.push_str(&current_line);
                full_text.push('\n');
                current_line.clear();
            }
            last_line_id = this_line_id;
        }

        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }

    if !current_line.is_empty() {
        full_text.push_str(&current_line);
        full_text.push('\n');
    }

    let mean_conf = if word_count > 0 {
        (total_confidence / word_count as f32).round()
    } else {
        0.0
    };

    (full_text, mean_conf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tsv() {
        let sample_tsv = "\
level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
1\t1\t0\t0\t0\t0\t0\t0\t250\t60\t-1\t
4\t1\t1\t1\t1\t0\t10\t12\t120\t7\t-1\t
5\t1\t1\t1\t1\t1\t10\t13\t41\t6\t95\tINVOICE
5\t1\t1\t1\t1\t2\t57\t13\t12\t6\t85\tNO:
5\t1\t1\t1\t1\t3\t82\t12\t48\t7\t90\t12345";

        let (text, conf) = parse_tsv(sample_tsv);
        assert_eq!(text.trim(), "INVOICE NO: 12345");
        assert_eq!(conf, 90.0);
    }
}
